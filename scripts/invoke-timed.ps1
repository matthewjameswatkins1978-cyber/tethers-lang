[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Label,
    [Parameter(Mandatory = $true)]
    [string]$Executable,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Arguments
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$sw = [System.Diagnostics.Stopwatch]::StartNew()

& $Executable @Arguments
$exitCode = $LASTEXITCODE

$sw.Stop()
$elapsedS = $sw.Elapsed.TotalSeconds
$elapsedMs = [int64]$sw.ElapsedMilliseconds

$status = if ($exitCode -eq 0) { 'PASS' } else { "FAIL($exitCode)" }
Write-Host "TIME $Label $('{0:F1}' -f $elapsedS)s $status"

try {
    $record = @{
        v          = 1
        at         = [DateTime]::UtcNow.ToString('yyyy-MM-ddTHH:mm:ssK')
        label      = $Label
        elapsed_ms = $elapsedMs
        exit_code  = $exitCode
    } | ConvertTo-Json -Compress

    $dir = '.tethers'
    if (-not (Test-Path -LiteralPath $dir)) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
    }

    Add-Content -LiteralPath '.tethers/timings.jsonl' -Value $record -Encoding utf8NoBOM
} catch {
    Write-Warning "invoke-timed: cannot write timing record: $_"
}

exit $exitCode
