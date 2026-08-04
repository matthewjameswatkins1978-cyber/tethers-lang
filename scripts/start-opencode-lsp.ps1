[CmdletBinding()]
param(
    [string]$OpenCodePath,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$OpenCodeArgs
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Resolve-OpenCodeApplication {
    param([string]$ExplicitPath)

    $candidate = if ($ExplicitPath) { $ExplicitPath } elseif ($env:OPENCODE_BIN) { $env:OPENCODE_BIN } else {
        $command = Get-Command opencode -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($command) { $command.Source } else { $null }
    }
    if (-not $candidate) { return $null }
    try { $item = Get-Item -LiteralPath $candidate -ErrorAction Stop } catch { return $null }
    if ($item.PSIsContainer -or $item.Extension.ToLowerInvariant() -notin @('.exe', '.cmd', '.bat', '.com')) { return $null }
    return $item.FullName
}

$hadLspTool = Test-Path Env:OPENCODE_EXPERIMENTAL_LSP_TOOL
$previousLspTool = if ($hadLspTool) { $env:OPENCODE_EXPERIMENTAL_LSP_TOOL } else { $null }
$hadDisableDownload = Test-Path Env:OPENCODE_DISABLE_LSP_DOWNLOAD
$previousDisableDownload = if ($hadDisableDownload) { $env:OPENCODE_DISABLE_LSP_DOWNLOAD } else { $null }

try {
    $application = Resolve-OpenCodeApplication -ExplicitPath $OpenCodePath
    if (-not $application) { throw 'OpenCode application not found; supply -OpenCodePath, set OPENCODE_BIN, or add opencode to PATH' }
    $env:OPENCODE_EXPERIMENTAL_LSP_TOOL = 'true'
    $env:OPENCODE_DISABLE_LSP_DOWNLOAD = 'true'
    $LASTEXITCODE = $null
    & $application @OpenCodeArgs
    if ($null -eq $LASTEXITCODE) { throw 'OpenCode did not return a console exit code' }
    $exitCode = $LASTEXITCODE
} catch {
    Write-Error $_.Exception.Message
    $exitCode = 1
} finally {
    if ($hadLspTool) { $env:OPENCODE_EXPERIMENTAL_LSP_TOOL = $previousLspTool } else { Remove-Item Env:OPENCODE_EXPERIMENTAL_LSP_TOOL -ErrorAction SilentlyContinue }
    if ($hadDisableDownload) { $env:OPENCODE_DISABLE_LSP_DOWNLOAD = $previousDisableDownload } else { Remove-Item Env:OPENCODE_DISABLE_LSP_DOWNLOAD -ErrorAction SilentlyContinue }
}

exit $exitCode
