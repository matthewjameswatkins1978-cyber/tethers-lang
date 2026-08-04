[CmdletBinding()]
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$OpenCodeArgs
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$hadLspTool = Test-Path Env:OPENCODE_EXPERIMENTAL_LSP_TOOL
$prevLspTool = if ($hadLspTool) { $env:OPENCODE_EXPERIMENTAL_LSP_TOOL } else { $null }
$hadDisableDownload = Test-Path Env:OPENCODE_DISABLE_LSP_DOWNLOAD
$prevDisableDownload = if ($hadDisableDownload) { $env:OPENCODE_DISABLE_LSP_DOWNLOAD } else { $null }

try {
    $env:OPENCODE_EXPERIMENTAL_LSP_TOOL = "true"
    $env:OPENCODE_DISABLE_LSP_DOWNLOAD = "true"

    $opencodeCmd = Get-Command opencode -CommandType Application -ErrorAction Stop
    & $opencodeCmd.Source @OpenCodeArgs
    $exitCode = $LASTEXITCODE
} catch {
    Write-Host "ERROR: opencode not found on PATH: $_"
    $exitCode = 1
} finally {
    if ($hadLspTool) {
        $env:OPENCODE_EXPERIMENTAL_LSP_TOOL = $prevLspTool
    } else {
        Remove-Item Env:OPENCODE_EXPERIMENTAL_LSP_TOOL -ErrorAction SilentlyContinue
    }
    if ($hadDisableDownload) {
        $env:OPENCODE_DISABLE_LSP_DOWNLOAD = $prevDisableDownload
    } else {
        Remove-Item Env:OPENCODE_DISABLE_LSP_DOWNLOAD -ErrorAction SilentlyContinue
    }
}

exit $exitCode
