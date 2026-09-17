[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$python = Get-Command py -ErrorAction SilentlyContinue
$script = Join-Path $PSScriptRoot 'check-compatibility-corpus.py'
if ($null -ne $python) { & $python.Source -3 $script } else { & (Get-Command python -ErrorAction Stop).Source $script }
exit $LASTEXITCODE
