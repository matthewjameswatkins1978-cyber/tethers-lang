param(
    [ValidateSet("native", "windows-x64", "linux-x64-musl")]
    [string]$Target = "native"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$TethersRoot = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")
$PortableRoot = Join-Path $TethersRoot "portable-rust"
$Manifest = Join-Path $PortableRoot "Cargo.toml"
$Version = (Get-Content -Raw -LiteralPath (Join-Path $PortableRoot "VERSION")).Trim()

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "cargo is required to package Tethers Portable."
}

Push-Location $PortableRoot
try {
    $CargoArgs = @("build", "--release", "--locked", "--manifest-path", $Manifest)
    if ($Target -eq "windows-x64" -or ($Target -eq "native" -and $IsWindows)) {
        $env:RUSTFLAGS = "-C target-feature=+crt-static"
        $CargoArgs += @("--target", "x86_64-pc-windows-msvc")
    } elseif ($Target -eq "linux-x64-musl") {
        $CargoArgs += @("--target", "x86_64-unknown-linux-musl")
    }

    & cargo @CargoArgs
    if ($LASTEXITCODE -ne 0) {
        throw "portable release build failed with exit code $LASTEXITCODE"
    }
}
finally {
    Pop-Location
}

$Rid = if ($Target -eq "windows-x64" -or ($Target -eq "native" -and $IsWindows)) {
    "windows-x64"
} elseif ($Target -eq "linux-x64-musl") {
    "linux-x64-musl"
} else {
    "linux-x64"
}
$BinaryName = if ($Rid -eq "windows-x64") { "tethers.exe" } else { "tethers" }
$BinaryRelative = if ($Rid -eq "windows-x64") {
    Join-Path (Join-Path "target" "x86_64-pc-windows-msvc") (Join-Path "release" $BinaryName)
} elseif ($Rid -eq "linux-x64-musl") {
    Join-Path (Join-Path "target" "x86_64-unknown-linux-musl") (Join-Path "release" $BinaryName)
} else {
    Join-Path (Join-Path "target" "release") $BinaryName
}
$Binary = Join-Path $PortableRoot $BinaryRelative
if (-not (Test-Path -LiteralPath $Binary -PathType Leaf)) {
    throw "release executable was not found at $Binary"
}

$DistRoot = Join-Path $TethersRoot "dist"
$Stage = Join-Path $DistRoot "tethers-portable-$Version-$Rid"
$Zip = Join-Path $DistRoot "tethers-portable-$Version-$Rid.zip"

if (Test-Path -LiteralPath $Stage) {
    Remove-Item -LiteralPath $Stage -Recurse -Force
}
if (Test-Path -LiteralPath $Zip) {
    Remove-Item -LiteralPath $Zip -Force
}
New-Item -ItemType Directory -Path $Stage -Force | Out-Null
Copy-Item -LiteralPath $Binary -Destination (Join-Path $Stage $BinaryName)
Copy-Item -LiteralPath (Join-Path $PortableRoot "policies") -Destination (Join-Path $Stage "policies") -Recurse
Copy-Item -LiteralPath (Join-Path $PortableRoot "examples") -Destination (Join-Path $Stage "examples") -Recurse
Copy-Item -LiteralPath (Join-Path $PortableRoot "README.md") -Destination (Join-Path $Stage "README.md")
Copy-Item -LiteralPath (Join-Path $PortableRoot "VERSION") -Destination (Join-Path $Stage "VERSION")

$Hash = Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $Stage $BinaryName)
"$($Hash.Hash.ToLowerInvariant())  $BinaryName" | Set-Content -LiteralPath (Join-Path $Stage "SHA256SUMS") -Encoding ascii
Compress-Archive -Path (Join-Path $Stage "*") -DestinationPath $Zip -CompressionLevel Optimal
Write-Output $Zip
