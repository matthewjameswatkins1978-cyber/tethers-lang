param(
    [ValidateSet(
        "valid", "changed-description", "missing-tool", "duplicate-tool",
        "wrong-tool", "input-schema-mismatch", "output-schema-mismatch",
        "initialization-error", "incompatible-version", "server-name-mismatch",
        "malformed-json", "exit-early",
        "hang-initialize", "hang-tools-list", "stdout-log-text",
        "oversized-line", "retained-stderr", "descendant-alive",
        "paginated-tools", "cursor-loop", "paged-duplicate",
        "catalogue-change-unchanged", "catalogue-change-drift",
        "catalogue-change-on-probe",
        "record-methods", "record-cwd", "run-success", "run-hang-initialize",
        "run-explicit-error", "run-invalid-output", "run-hang-call",
        "c2-overlap-barrier"
    )]
    [string]$Mode = "valid",
    [string]$MarkerFile = "",
    [string]$CwdMarkerFile = "",
    [string]$BarrierDirectory = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$stderr = [System.Console]::Error
$protocolVersion = "2025-11-25"
$initialized = $false
$clientInitialized = $false
$toolsListCount = 0

function Write-JsonLine {
    param([Parameter(Mandatory = $true)]$Object)
    [System.Console]::Out.WriteLine(($Object | ConvertTo-Json -Compress -Depth 30))
}

function Write-ErrorResponse {
    param($Id, [int]$Code, [string]$Message)
    Write-JsonLine @{ jsonrpc = "2.0"; id = $Id; error = @{ code = $Code; message = $Message } }
}

function New-Tool {
    $name = if ($Mode -eq "wrong-tool") { "fixture_other" } else { "fixture_ping" }
    $description = if ($Mode -eq "changed-description") {
        "Provider-controlled description changed."
    } else {
        "Echo one message for deterministic provider binding tests."
    }
    $inputSchema = @{
        type = "object"
        properties = @{ message = @{ type = "string" } }
        required = @("message")
        additionalProperties = $false
    }
    if ($Mode -in @("run-success", "run-explicit-error", "run-invalid-output", "run-hang-call")) {
        $inputSchema.properties.path = @{ type = "string" }
        $inputSchema.required = @("message", "path")
    }
    if ($Mode -eq "input-schema-mismatch") { $inputSchema.required = @("different") }
    $outputSchema = @{
        type = "object"
        properties = @{ echo = @{ type = "string" } }
        required = @("echo")
        additionalProperties = $false
    }
    if ($Mode -eq "output-schema-mismatch") { $outputSchema.required = @("different") }
    return @{
        name = $name; description = $description
        inputSchema = $inputSchema; outputSchema = $outputSchema
    }
}

# --- Special modes that bypass the main loop ---

if ($Mode -eq "hang-initialize") {
    $stderr.WriteLine("fixture: hanging during initialize")
    while ($true) { Start-Sleep -Seconds 60 }
}
if ($Mode -eq "run-hang-initialize") {
    if ($MarkerFile) { Add-Content -Path $MarkerFile -Value "provider_started" }
    $stderr.WriteLine("fixture: hanging during run initialization")
    while ($true) { Start-Sleep -Seconds 60 }
}
if ($Mode -eq "stdout-log-text") {
    $stderr.WriteLine("fixture: emitting log text on stdout")
    [System.Console]::Out.WriteLine("LOG: this is not JSON")
}
if ($Mode -eq "oversized-line") {
    $stderr.WriteLine("fixture: emitting oversized protocol line")
    $big = "x" * (9 * 1024 * 1024)
    [System.Console]::Out.WriteLine($big)
}
if ($Mode -eq "retained-stderr") {
    $stderr.WriteLine("fixture: writing diagnostic to stderr")
    $stderr.WriteLine("fixture: additional diagnostic data")
}
if ($Mode -eq "descendant-alive") {
    $stderr.WriteLine("fixture: spawning descendant process")
    $d = Start-Process -FilePath "pwsh.exe" -ArgumentList "-NoProfile", "-Command", "Start-Sleep -Seconds 300" -PassThru -WindowStyle Hidden
    $stderr.WriteLine("fixture: descendant PID=$($d.Id)")
}
if ($Mode -eq "record-cwd" -and $CwdMarkerFile) {
    Set-Content -Path $CwdMarkerFile -Value (Get-Location).Path
    $stderr.WriteLine("fixture: recorded CWD to $CwdMarkerFile")
}
if ($Mode -eq "exit-early") {
    $stderr.WriteLine("fixture: exiting before initialization")
    $stderr.Flush()
    Start-Sleep -Milliseconds 50
    exit 0
}

# --- Main MCP protocol loop ---

$reader = [System.IO.StreamReader]::new([System.Console]::OpenStandardInput())

try {
    while ($true) {
        $line = $reader.ReadLine()
        if ($null -eq $line) { break }
        if ($line.Trim().Length -eq 0) { continue }

        $request = $null
        try { $request = $line | ConvertFrom-Json -ErrorAction Stop }
        catch { $stderr.WriteLine("fixture: malformed JSON input"); continue }

        if ($request.jsonrpc -ne "2.0" -or $request.method -isnot [string]) {
            $id = if ($null -ne $request.PSObject.Properties["id"]) { $request.id } else { $null }
            Write-ErrorResponse $id -32600 "Invalid Request"
            continue
        }

        switch ($request.method) {
            "initialize" {
                if ($Mode -in @("record-methods", "run-success", "run-explicit-error", "run-invalid-output", "run-hang-call", "missing-tool") -and $MarkerFile) {
                    Add-Content -Path $MarkerFile -Value "initialize"
                }
                if ($Mode -eq "malformed-json") {
                    [System.Console]::Out.WriteLine("{not-json")
                    continue
                }
                if ($Mode -eq "initialization-error") {
                    Write-ErrorResponse $request.id -32602 "Initialization rejected by fixture"
                    continue
                }
                $selectedVersion = if ($Mode -eq "incompatible-version") { "1900-01-01" } else { $protocolVersion }
                $serverName = if ($Mode -eq "server-name-mismatch") { "unexpected-provider" } else { "tethers-stdio-fixture" }
                $initialized = $true
                Write-JsonLine @{
                    jsonrpc = "2.0"; id = $request.id
                    result = @{
                        protocolVersion = $selectedVersion
                        capabilities = @{ tools = @{} }
                        serverInfo = @{ name = $serverName; version = "0.1.0" }
                    }
                }
            }
            "notifications/initialized" {
                if (-not $initialized) {
                    $stderr.WriteLine("fixture: initialized notification before initialize")
                    continue
                }
                $clientInitialized = $true
            }
            "tools/list" {
                $toolsListCount += 1
                if ($Mode -in @("record-methods", "run-success", "run-explicit-error", "run-invalid-output", "run-hang-call", "missing-tool") -and $MarkerFile) {
                    Add-Content -Path $MarkerFile -Value "tools/list"
                }
                if ($Mode -eq "hang-tools-list") {
                    Start-Sleep -Seconds 3600
                }
                if (-not $clientInitialized) {
                    Write-ErrorResponse $request.id -32002 "Server not initialized"
                    continue
                }
                if ($Mode -in @("paginated-tools", "cursor-loop", "paged-duplicate")) {
                    $hasCursor = $null -ne $request.params.PSObject.Properties["cursor"]
                    if (-not $hasCursor) {
                        Write-JsonLine @{
                            jsonrpc = "2.0"; id = $request.id
                            result = @{ tools = @(New-Tool); nextCursor = "opaque::+/=" }
                        }
                        continue
                    }
                    if ($request.params.cursor -ne "opaque::+/=") {
                        Write-ErrorResponse $request.id -32602 "opaque cursor was changed"
                        continue
                    }
                    [object[]]$pageTools = if ($Mode -eq "paged-duplicate") {
                        @(New-Tool)
                    } else {
                        @(@{
                            name = "fixture_unapproved_addition"
                            description = "Untrusted additional operation."
                            inputSchema = @{ type = "object" }
                            outputSchema = @{ type = "object" }
                            annotations = @{ readOnlyHint = $true }
                        })
                    }
                    $result = @{ tools = $pageTools }
                    if ($Mode -eq "cursor-loop") { $result.nextCursor = "opaque::+/=" }
                    Write-JsonLine @{ jsonrpc = "2.0"; id = $request.id; result = $result }
                    continue
                }

                $tools = @()
                if ($Mode -ne "missing-tool") { $tools += New-Tool }
                if ($Mode -eq "duplicate-tool") { $tools += New-Tool }
                if ($Mode -eq "catalogue-change-drift" -and $toolsListCount -gt 1) {
                    $tools[0].inputSchema.required = @("different")
                }
                if ($Mode -in @("catalogue-change-unchanged", "catalogue-change-drift") -and $toolsListCount -eq 1) {
                    Write-JsonLine @{ jsonrpc = "2.0"; method = "notifications/tools/list_changed"; params = @{} }
                }
                Write-JsonLine @{
                    jsonrpc = "2.0"; id = $request.id
                    result = @{ tools = $tools }
                }
            }
            "tools/call" {
                if ($Mode -eq "c2-overlap-barrier") {
                    if ([string]::IsNullOrWhiteSpace($BarrierDirectory)) {
                        Write-ErrorResponse $request.id -32602 "BarrierDirectory is required"
                        continue
                    }
                    [System.IO.Directory]::CreateDirectory($BarrierDirectory) | Out-Null
                    $token = "$PID-$([guid]::NewGuid().ToString('N'))"
                    $entered = Join-Path $BarrierDirectory "entered-$token"
                    $active = Join-Path $BarrierDirectory "active-$token"
                    [System.IO.File]::WriteAllText($entered, "entered")
                    $limit = [DateTime]::UtcNow.AddSeconds(10)
                    while ((Get-ChildItem -LiteralPath $BarrierDirectory -Filter 'entered-*').Count -lt 2) {
                        if ([DateTime]::UtcNow -gt $limit) {
                            Write-ErrorResponse $request.id -32000 "overlap peer did not enter"
                            continue 2
                        }
                        Start-Sleep -Milliseconds 10
                    }
                    [System.IO.File]::WriteAllText($active, "active")
                    while (-not (Test-Path -LiteralPath (Join-Path $BarrierDirectory 'release'))) {
                        if ([DateTime]::UtcNow -gt $limit) {
                            Write-ErrorResponse $request.id -32000 "overlap release timed out"
                            continue 2
                        }
                        Start-Sleep -Milliseconds 10
                    }
                    $message = $request.params.arguments.message
                    Write-JsonLine @{ jsonrpc = "2.0"; id = $request.id; result = @{ echo = $message } }
                    continue
                }
                if ($Mode -eq "run-success") {
                    if ($MarkerFile) { Add-Content -Path $MarkerFile -Value "tools/call" }
                    $message = $request.params.arguments.message
                    if ($message -isnot [string]) {
                        Write-ErrorResponse $request.id -32602 "fixture_ping requires string message"
                        continue
                    }
                    Write-JsonLine @{
                        jsonrpc = "2.0"; id = $request.id
                        result = @{ echo = $message }
                    }
                    continue
                }
                if ($Mode -eq "run-explicit-error") {
                    if ($MarkerFile) { Add-Content -Path $MarkerFile -Value "tools/call" }
                    Write-ErrorResponse $request.id -32600 "fixture explicit error for negative matrix"
                    continue
                }
                if ($Mode -eq "run-invalid-output") {
                    if ($MarkerFile) { Add-Content -Path $MarkerFile -Value "tools/call" }
                    Write-JsonLine @{
                        jsonrpc = "2.0"; id = $request.id
                        result = @{ wrong_field = "not echo" }
                    }
                    continue
                }
                if ($Mode -eq "run-hang-call") {
                    if ($MarkerFile) { Add-Content -Path $MarkerFile -Value "tools/call" }
                    $stderr.WriteLine("fixture: hanging during tools/call")
                    while ($true) { Start-Sleep -Seconds 60 }
                }
                if ($Mode -eq "record-methods" -and $MarkerFile) {
                    Add-Content -Path $MarkerFile -Value "tools/call"
                }
                Write-ErrorResponse $request.id -32601 "Method not found"
            }
            "ping" {
                if ($Mode -eq "catalogue-change-on-probe") {
                    Write-JsonLine @{ jsonrpc = "2.0"; method = "notifications/tools/list_changed"; params = @{} }
                }
                Write-JsonLine @{ jsonrpc = "2.0"; id = $request.id; result = @{} }
            }
            default {
                if ($Mode -eq "record-methods" -and $MarkerFile) {
                    Add-Content -Path $MarkerFile -Value $request.method
                }
                $id = if ($null -ne $request.PSObject.Properties["id"]) { $request.id } else { $null }
                Write-ErrorResponse $id -32601 "Method not found"
            }
        }
    }
}
finally {
    $reader.Dispose()
}
