Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$EngineDir = Join-Path $PSScriptRoot ".." "engine-ocaml"
$ServerExe = Join-Path $EngineDir "_build" "default" "bin" "tethers_mcp_main.exe"

Push-Location $EngineDir
try {
    opam exec -- $ServerExe
}
finally {
    Pop-Location
}