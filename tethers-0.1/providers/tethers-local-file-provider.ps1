# Tethers local file provider.
#
# A dedicated stdio MCP provider exposing exactly one real capability tool,
# `file_move`.  It is not a test fixture and has no failure-injection modes:
# every rejection below corresponds to a genuine safety boundary.
#
# Trust boundary.  The host supplies -ProviderRoot, -SourcePrefix and
# -DestinationPrefix as trusted runtime configuration.  Tool arguments arrive
# from the untrusted protocol channel and are validated here independently of
# any host-side scope assessment.  The accepted host runtime binds `path_prefix`
# scope to a single JSON pointer (`/source_path`), so destination confinement
# is owned solely by this provider.
#
# The provider performs no watching, scanning, polling, network access, or
# daemon behaviour.  It reads one request line, answers it, and blocks on the
# next line until stdin closes.

param(
    [Parameter(Mandatory = $true)]
    [string]$ProviderRoot,

    [Parameter(Mandatory = $true)]
    [string]$SourcePrefix,

    [Parameter(Mandatory = $true)]
    [string]$DestinationPrefix,

    [string]$MarkerFile = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$stderr = [System.Console]::Error
$protocolVersion = "2025-11-25"
$serverName = "tethers-local-file-provider"
$toolName = "file_move"

# JSON-RPC reserved codes used for the two honest failure classes:
# -32602 for a rejected argument, -32603 for a filesystem operation failure.
$InvalidParams = -32602
$InternalError = -32603

$initialized = $false
$clientInitialized = $false

function Write-JsonLine {
    param([Parameter(Mandatory = $true)]$Object)
    [System.Console]::Out.WriteLine(($Object | ConvertTo-Json -Compress -Depth 30))
}

function Write-ErrorResponse {
    param($Id, [int]$Code, [string]$Message)
    Write-JsonLine @{ jsonrpc = "2.0"; id = $Id; error = @{ code = $Code; message = $Message } }
}

function Add-Marker {
    param([string]$Method)
    if ($MarkerFile) { Add-Content -LiteralPath $MarkerFile -Value $Method }
}

# ---------------------------------------------------------------------------
# Trusted configuration
# ---------------------------------------------------------------------------

function Get-CanonicalDirectory {
    param([string]$Path)
    $item = Get-Item -LiteralPath $Path -Force
    if ($item -isnot [System.IO.DirectoryInfo]) {
        throw "provider root is not a directory: $Path"
    }
    # GetFullPath normalises separators and relative segments; the trailing
    # separator is removed so containment comparison can enforce a segment
    # boundary explicitly.
    return [System.IO.Path]::GetFullPath($item.FullName).TrimEnd(
        [System.IO.Path]::DirectorySeparatorChar,
        [System.IO.Path]::AltDirectorySeparatorChar)
}

function Assert-ConfiguredPrefix {
    param([string]$Prefix, [string]$Label)
    if ([string]::IsNullOrEmpty($Prefix)) { throw "$Label must not be empty" }
    if (-not $Prefix.EndsWith("/")) { throw "$Label must end with '/'" }
    if ($Prefix.Contains("\")) { throw "$Label must not contain a backslash" }
    if ($Prefix.StartsWith("/")) { throw "$Label must be relative" }
    foreach ($segment in $Prefix.TrimEnd("/").Split("/")) {
        if ($segment -eq "" -or $segment -eq "." -or $segment -eq ".." -or $segment.Contains(":")) {
            throw "$Label contains an unsafe segment"
        }
    }
}

try {
    if (-not (Test-Path -LiteralPath $ProviderRoot)) {
        throw "provider root does not exist: $ProviderRoot"
    }
    $canonicalRoot = Get-CanonicalDirectory $ProviderRoot
    Assert-ConfiguredPrefix $SourcePrefix "SourcePrefix"
    Assert-ConfiguredPrefix $DestinationPrefix "DestinationPrefix"
}
catch {
    $stderr.WriteLine("tethers-local-file-provider: configuration rejected: $($_.Exception.Message)")
    exit 2
}

# ---------------------------------------------------------------------------
# Resource-path validation
# ---------------------------------------------------------------------------

# A resource path is a relative forward-slash path beneath the provider root.
# Every rejection here is a refusal to act, never a repaired path: a caller
# that supplies something ambiguous gets an error, not a guess.
function Test-ResourcePath {
    param([string]$Path, [string]$Label)

    if ([string]::IsNullOrEmpty($Path)) { return "$Label must not be empty" }
    if ($Path.Contains([char]0)) { return "$Label must not contain NUL" }
    if ($Path.Contains("\")) { return "$Label must use '/' and must not contain a backslash" }
    if ($Path.Contains(":")) { return "$Label must not contain ':'" }
    if ($Path.StartsWith("/")) { return "$Label must be relative, not rooted" }
    if ([System.IO.Path]::IsPathRooted($Path)) { return "$Label must not be an absolute or drive-relative path" }

    foreach ($segment in $Path.Split("/")) {
        if ($segment -eq "") { return "$Label must not contain empty segments" }
        if ($segment -eq "." -or $segment -eq "..") { return "$Label must not contain '.' or '..' segments" }
    }
    return $null
}

function Test-HasPrefix {
    param([string]$Path, [string]$Prefix)
    return $Path.StartsWith($Prefix, [System.StringComparison]::OrdinalIgnoreCase)
}

# Windows path comparison is case-insensitive.  Containment additionally
# requires a directory-separator boundary so that a sibling directory such as
# "root-elsewhere" can never be mistaken for a child of "root".
function Test-WithinRoot {
    param([string]$FullPath, [string]$Root)
    if ($FullPath.Equals($Root, [System.StringComparison]::OrdinalIgnoreCase)) { return $false }
    $prefix = $Root + [System.IO.Path]::DirectorySeparatorChar
    return $FullPath.StartsWith($prefix, [System.StringComparison]::OrdinalIgnoreCase)
}

# Reparse points (junctions, symbolic links, mount points) are the one escape
# route that pure string validation cannot see, because the escaping component
# may itself be a legitimate-looking name beneath the root.  Every existing
# component from the root down to the candidate is therefore inspected.
function Get-ReparseEscape {
    param([string]$FullPath, [string]$Root)

    $relative = $FullPath.Substring($Root.Length).TrimStart([System.IO.Path]::DirectorySeparatorChar)
    if ($relative -eq "") { return $null }

    $current = $Root
    foreach ($segment in $relative.Split([System.IO.Path]::DirectorySeparatorChar)) {
        $current = [System.IO.Path]::Combine($current, $segment)
        $info = $null
        if ([System.IO.Directory]::Exists($current)) {
            $info = [System.IO.DirectoryInfo]::new($current)
        }
        elseif ([System.IO.File]::Exists($current)) {
            $info = [System.IO.FileInfo]::new($current)
        }
        if ($null -ne $info -and
            ($info.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
            return $current
        }
    }
    return $null
}

# Resolve one validated resource path to an absolute path that is proved to
# stay inside the canonical provider root, both lexically and after reparse
# inspection.  Returns the absolute path or throws the caller-facing message.
function Resolve-ResourcePath {
    param([string]$Path, [string]$Label)

    $message = Test-ResourcePath $Path $Label
    if ($null -ne $message) { throw $message }

    $combined = [System.IO.Path]::Combine($canonicalRoot, ($Path -replace "/", [System.IO.Path]::DirectorySeparatorChar))
    $full = [System.IO.Path]::GetFullPath($combined)

    if (-not (Test-WithinRoot $full $canonicalRoot)) {
        throw "$Label resolves outside the provider root"
    }

    $reparse = Get-ReparseEscape $full $canonicalRoot
    if ($null -ne $reparse) {
        throw "$Label passes through a reparse point and is refused"
    }

    return $full
}

# ---------------------------------------------------------------------------
# file_move
# ---------------------------------------------------------------------------

function New-ToolDescriptor {
    $inputSchema = [ordered]@{
        type = "object"
        properties = [ordered]@{
            source_path = @{ type = "string" }
            destination_path = @{ type = "string" }
        }
        required = @("source_path", "destination_path")
        additionalProperties = $false
    }
    $outputSchema = [ordered]@{
        type = "object"
        properties = [ordered]@{
            moved = @{ type = "boolean" }
            source_path = @{ type = "string" }
            destination_path = @{ type = "string" }
        }
        required = @("moved", "source_path", "destination_path")
        additionalProperties = $false
    }
    return [ordered]@{
        name = $toolName
        description = "Move one existing file between two configured directories beneath the provider root."
        inputSchema = $inputSchema
        outputSchema = $outputSchema
    }
}

# Perform the move or throw.  Every precondition is checked before the single
# effectful call, and the success response is written only after the move has
# been observed to have taken effect.
function Invoke-FileMove {
    param([string]$SourceResource, [string]$DestinationResource)

    if (-not (Test-HasPrefix $SourceResource $SourcePrefix)) {
        throw "source_path is not beneath the configured source prefix"
    }
    if (-not (Test-HasPrefix $DestinationResource $DestinationPrefix)) {
        throw "destination_path is not beneath the configured destination prefix"
    }

    $sourceFull = Resolve-ResourcePath $SourceResource "source_path"
    $destinationFull = Resolve-ResourcePath $DestinationResource "destination_path"

    if ($sourceFull.Equals($destinationFull, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "source_path and destination_path resolve to the same file"
    }
    if (-not [System.IO.File]::Exists($sourceFull)) {
        throw "source_path does not exist as a regular file"
    }
    if ([System.IO.Directory]::Exists($destinationFull)) {
        throw "destination_path already exists as a directory"
    }
    if ([System.IO.File]::Exists($destinationFull)) {
        throw "destination_path already exists and is never overwritten"
    }

    $destinationParent = [System.IO.Path]::GetDirectoryName($destinationFull)
    if (-not [System.IO.Directory]::Exists($destinationParent)) {
        throw "destination_path parent directory does not exist and is never created"
    }

    # Literal .NET move with overwrite explicitly disabled.  Cmdlets such as
    # Move-Item are avoided because they expand wildcards in the path.
    [System.IO.File]::Move($sourceFull, $destinationFull, $false)

    if ([System.IO.File]::Exists($sourceFull) -or -not [System.IO.File]::Exists($destinationFull)) {
        throw "move did not complete as expected"
    }

    return [ordered]@{
        moved = $true
        source_path = $SourceResource
        destination_path = $DestinationResource
    }
}

# ---------------------------------------------------------------------------
# MCP protocol loop
# ---------------------------------------------------------------------------

$reader = [System.IO.StreamReader]::new([System.Console]::OpenStandardInput())

try {
    while ($true) {
        $line = $reader.ReadLine()
        if ($null -eq $line) { break }
        if ($line.Trim().Length -eq 0) { continue }

        $request = $null
        try { $request = $line | ConvertFrom-Json -ErrorAction Stop }
        catch {
            $stderr.WriteLine("tethers-local-file-provider: malformed JSON input")
            continue
        }

        if ($request.jsonrpc -ne "2.0" -or $request.method -isnot [string]) {
            $id = if ($null -ne $request.PSObject.Properties["id"]) { $request.id } else { $null }
            Write-ErrorResponse $id -32600 "Invalid Request"
            continue
        }

        switch ($request.method) {
            "initialize" {
                Add-Marker "initialize"
                $initialized = $true
                Write-JsonLine @{
                    jsonrpc = "2.0"; id = $request.id
                    result = @{
                        protocolVersion = $protocolVersion
                        capabilities = @{ tools = @{} }
                        serverInfo = @{ name = $serverName; version = "0.1.0" }
                    }
                }
            }
            "notifications/initialized" {
                if (-not $initialized) {
                    $stderr.WriteLine("tethers-local-file-provider: initialized notification before initialize")
                    continue
                }
                $clientInitialized = $true
            }
            "tools/list" {
                Add-Marker "tools/list"
                if (-not $clientInitialized) {
                    Write-ErrorResponse $request.id -32002 "Server not initialized"
                    continue
                }
                Write-JsonLine @{
                    jsonrpc = "2.0"; id = $request.id
                    result = @{ tools = @(New-ToolDescriptor) }
                }
            }
            "tools/call" {
                Add-Marker "tools/call"
                if (-not $clientInitialized) {
                    Write-ErrorResponse $request.id -32002 "Server not initialized"
                    continue
                }

                $name = $request.params.name
                if ($name -ne $toolName) {
                    Write-ErrorResponse $request.id -32601 "unknown tool '$name'"
                    continue
                }

                $arguments = $request.params.arguments
                $sourceResource = $null
                $destinationResource = $null
                if ($null -ne $arguments) {
                    if ($null -ne $arguments.PSObject.Properties["source_path"]) {
                        $sourceResource = $arguments.source_path
                    }
                    if ($null -ne $arguments.PSObject.Properties["destination_path"]) {
                        $destinationResource = $arguments.destination_path
                    }
                }
                if ($sourceResource -isnot [string] -or $destinationResource -isnot [string]) {
                    Write-ErrorResponse $request.id $InvalidParams `
                        "file_move requires string source_path and destination_path"
                    continue
                }

                try {
                    $result = Invoke-FileMove $sourceResource $destinationResource
                }
                catch [System.IO.IOException] {
                    $stderr.WriteLine("tethers-local-file-provider: move failed: $($_.Exception.Message)")
                    Write-ErrorResponse $request.id $InternalError "file_move failed: $($_.Exception.Message)"
                    continue
                }
                catch {
                    $stderr.WriteLine("tethers-local-file-provider: refused: $($_.Exception.Message)")
                    Write-ErrorResponse $request.id $InvalidParams "file_move refused: $($_.Exception.Message)"
                    continue
                }

                Write-JsonLine @{ jsonrpc = "2.0"; id = $request.id; result = $result }
            }
            default {
                $id = if ($null -ne $request.PSObject.Properties["id"]) { $request.id } else { $null }
                Write-ErrorResponse $id -32601 "Method not found"
            }
        }
    }
}
finally {
    $reader.Dispose()
}
