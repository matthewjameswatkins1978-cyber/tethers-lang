[CmdletBinding()]
param(
    [string]$OcamlSwitchPath,
    [switch]$ReleaseMode
)

$ErrorActionPreference = 'Stop'
$script = Join-Path $PSScriptRoot 'verify-tethers.py'
$arguments = @($script)
if (-not [string]::IsNullOrWhiteSpace($OcamlSwitchPath)) { $arguments += @('--ocaml-switch', $OcamlSwitchPath) }
if ($ReleaseMode) { $arguments += '--release' }
$python = Get-Command py -ErrorAction SilentlyContinue
if ($null -ne $python) { & $python.Source -3 @arguments } else { & (Get-Command python -ErrorAction Stop).Source @arguments }
exit $LASTEXITCODE
