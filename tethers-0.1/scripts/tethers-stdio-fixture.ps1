param(
    [ValidateSet(
        "valid",
        "changed-description",
        "missing-tool",
        "duplicate-tool",
        "wrong-tool",
        "input-schema-mismatch",
        "output-schema-mismatch",
        "initialization-error",
        "incompatible-version",
        "server-name-mismatch",
        "malformed-json",
        "exit-early"
    )]
    [string]$Mode = "valid"
)

# Deterministic test-only MCP provider fixture.
#
# Protocol is newline-delimited JSON-RPC 2.0 over stdio. Protocol messages are
# written only to stdout and diagnostics only to stderr. The fixture advertises
# untrusted MCP tool metadata; the trusted Tethers manifest lives separately
# under protocol/capability-manifests.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$stderr = [System.Console]::Error
$protocolVersion = "2025-11-25"
$initialized = $false
$clientInitialized = $false

function Write-JsonLine {
    param([Parameter(Mandatory = $true)]$Object)

    [System.Console]::Out.WriteLine(
        ($Object | ConvertTo-Json -Compress -Depth 30)
    )
}

function Write-ErrorResponse {
    param(
        $Id,
        [int]$Code,
        [string]$Message
    )

    Write-JsonLine @{
        jsonrpc = "2.0"
        id = $Id
        error = @{
            code = $Code
            message = $Message
        }
    }
}

function New-Tool {
    $name = if ($Mode -eq "wrong-tool") { "fixture_other" } else { "fixture_ping" }
    $description = if ($Mode -eq "changed-description") {
        "Provider-controlled description changed."
    }
    else {
        "Echo one message for deterministic provider binding tests."
    }

    $inputSchema = @{
        type = "object"
        properties = @{
            message = @{ type = "string" }
        }
        required = @("message")
        additionalProperties = $false
    }
    if ($Mode -eq "input-schema-mismatch") {
        $inputSchema.required = @("different")
    }

    $outputSchema = @{
        type = "object"
        properties = @{
            echo = @{ type = "string" }
        }
        required = @("echo")
        additionalProperties = $false
    }
    if ($Mode -eq "output-schema-mismatch") {
        $outputSchema.required = @("different")
    }

    return @{
        name = $name
        description = $description
        inputSchema = $inputSchema
        outputSchema = $outputSchema
    }
}

if ($Mode -eq "exit-early") {
    $stderr.WriteLine("fixture: exiting before initialization")
    exit 0
}

$reader = [System.IO.StreamReader]::new([System.Console]::OpenStandardInput())

try {
    while ($true) {
        $line = $reader.ReadLine()
        if ($null -eq $line) {
            break
        }
        if ($line.Trim().Length -eq 0) {
            continue
        }

        $request = $null
        try {
            $request = $line | ConvertFrom-Json -ErrorAction Stop
        }
        catch {
            $stderr.WriteLine("fixture: malformed JSON input")
            continue
        }

        if ($request.jsonrpc -ne "2.0" -or $request.method -isnot [string]) {
            $id = if ($null -ne $request.PSObject.Properties["id"]) { $request.id } else { $null }
            Write-ErrorResponse $id -32600 "Invalid Request"
            continue
        }

        @"
switch ($request.method) {
            "initialize" {
                if ($Mode -eq "record-methods" -and $MarkerFile) {
                    Add-Content -Path $MarkerFile -Value "initialize"
                }
"@
            "initialize" {
                if ($Mode -eq "malformed-json") {
                    [System.Console]::Out.WriteLine("{not-json")
                    continue
                }
                if ($Mode -eq "initialization-error") {
                    Write-ErrorResponse $request.id -32602 "Initialization rejected by fixture"
                    continue
                }

                $selectedVersion = if ($Mode -eq "incompatible-version") {
                    "1900-01-01"
                }
                else {
                    $protocolVersion
                }
                $serverName = if ($Mode -eq "server-name-mismatch") {
                    "unexpected-provider"
                }
                else {
                    "tethers-stdio-fixture"
                }

                $initialized = $true
                Write-JsonLine @{
                    jsonrpc = "2.0"
                    id = $request.id
                    result = @{
                        protocolVersion = $selectedVersion
                        capabilities = @{
                            tools = @{}
                        }
                        serverInfo = @{
                            name = $serverName
                            version = "0.1.0"
                        }
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
            @"
"tools/list" {
                if ($Mode -eq "hang-tools-list") {
                    Start-Sleep -Seconds 3600
                }
                if ($Mode -eq "record-methods" -and $MarkerFile) {
                    Add-Content -Path $MarkerFile -Value "tools/list"
                }
"@
                if (-not $clientInitialized) {
                    Write-ErrorResponse $request.id -32002 "Server not initialized"
                    continue
                }

                $tools = @()
                if ($Mode -ne "missing-tool") {
                    $tools += New-Tool
                }
                if ($Mode -eq "duplicate-tool") {
                    $tools += New-Tool
                }

                Write-JsonLine @{
                    jsonrpc = "2.0"
                    id = $request.id
                    result = @{
                        tools = $tools
                    }
                }
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
