param(
  [ValidateSet('windows-x64','linux-x64-musl')]
  [string]$Target = 'windows-x64'
)

$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$version = (Get-Content (Join-Path $root 'VERSION') -Raw).Trim()
$cargoTarget = if ($Target -eq 'windows-x64') { 'x86_64-pc-windows-msvc' } else { 'x86_64-unknown-linux-musl' }
$extension = if ($Target -eq 'windows-x64') { '.exe' } else { '' }
$dist = Join-Path $root 'dist'
$stage = Join-Path $dist "tethers-portable-$version-$Target"
$zip = Join-Path $dist "tethers-portable-$version-$Target.zip"

New-Item -ItemType Directory -Force -Path $dist | Out-Null
if (Test-Path -LiteralPath $stage) { Remove-Item -LiteralPath $stage -Recurse -Force }
if (Test-Path -LiteralPath $zip) { Remove-Item -LiteralPath $zip -Force }
New-Item -ItemType Directory -Force -Path (Join-Path $stage 'bin'), (Join-Path $stage 'policies'), (Join-Path $stage 'registry'), (Join-Path $stage 'plugs'), (Join-Path $stage 'docs'), (Join-Path $stage 'tests'), (Join-Path $stage 'scripts'), (Join-Path $stage 'schemas'), (Join-Path $stage 'examples'), (Join-Path $stage 'wrappers/rust'), (Join-Path $stage 'wrappers/go'), (Join-Path $stage 'wrappers/typescript'), (Join-Path $stage 'wrappers/python') | Out-Null

cargo build --release --locked --target $cargoTarget --manifest-path (Join-Path $root 'Cargo.toml')
$binary = Join-Path $root "target/$cargoTarget/release/tethers$extension"
if (-not (Test-Path -LiteralPath $binary)) { throw "build did not produce $binary" }
Copy-Item -LiteralPath $binary -Destination (Join-Path $stage "bin/tethers$extension")
Copy-Item -LiteralPath (Join-Path $root 'policies/default.json') -Destination (Join-Path $stage 'policies/workbench-default.json')
Copy-Item -Path (Join-Path $root 'policies/*.json') -Destination (Join-Path $stage 'policies')
Copy-Item -Path (Join-Path $root 'registry/*') -Destination (Join-Path $stage 'registry')
Copy-Item -Path (Join-Path $root 'plugs/*') -Destination (Join-Path $stage 'plugs')
Copy-Item -Path (Join-Path $root 'docs/*') -Destination (Join-Path $stage 'docs')
Copy-Item -Path (Join-Path $root 'tests/*') -Destination (Join-Path $stage 'tests')
Copy-Item -LiteralPath (Join-Path $root 'scripts/run_parity.py') -Destination (Join-Path $stage 'scripts')
Copy-Item -LiteralPath (Join-Path $root 'scripts/benchmark.py') -Destination (Join-Path $stage 'scripts')
Copy-Item -LiteralPath (Join-Path $root 'scripts/package-portable.py') -Destination (Join-Path $stage 'scripts')
Copy-Item -Path (Join-Path $root 'schemas/*') -Destination (Join-Path $stage 'schemas')
Copy-Item -Path (Join-Path $root 'examples/*') -Destination (Join-Path $stage 'examples')
Copy-Item -LiteralPath (Join-Path $root 'wrappers/rust/Cargo.toml') -Destination (Join-Path $stage 'wrappers/rust')
Copy-Item -LiteralPath (Join-Path $root 'wrappers/rust/Cargo.lock') -Destination (Join-Path $stage 'wrappers/rust')
Copy-Item -LiteralPath (Join-Path $root 'wrappers/rust/src') -Destination (Join-Path $stage 'wrappers/rust') -Recurse
Copy-Item -LiteralPath (Join-Path $root 'wrappers/rust/tests') -Destination (Join-Path $stage 'wrappers/rust') -Recurse
Copy-Item -Path (Join-Path $root 'wrappers/go/*') -Destination (Join-Path $stage 'wrappers/go')
Copy-Item -Path (Join-Path $root 'wrappers/typescript/*') -Destination (Join-Path $stage 'wrappers/typescript')
Copy-Item -Path (Join-Path $root 'wrappers/python/*') -Destination (Join-Path $stage 'wrappers/python')
Copy-Item -LiteralPath (Join-Path $root 'README.md') -Destination $stage
Copy-Item -LiteralPath (Join-Path $root 'RELEASE.md') -Destination $stage
Copy-Item -LiteralPath (Join-Path $root 'QUICKSTART.md') -Destination $stage
Copy-Item -LiteralPath (Join-Path $root 'AI-INTEGRATION.md') -Destination $stage
Copy-Item -LiteralPath (Join-Path $root 'VERSION') -Destination $stage

$hash = (Get-FileHash (Join-Path $stage "bin/tethers$extension") -Algorithm SHA256).Hash.ToUpperInvariant()
"$hash  bin/tethers$extension" | Set-Content -NoNewline (Join-Path $stage 'SHA256SUMS')
$python = Get-Command python -ErrorAction SilentlyContinue
if (-not $python) { throw 'Python is required to create the deterministic ZIP' }
& $python.Source (Join-Path $root 'scripts/deterministic_zip.py') $stage $zip
if ($LASTEXITCODE -ne 0) { throw 'deterministic ZIP creation failed' }
$zipHash = (Get-FileHash $zip -Algorithm SHA256).Hash.ToUpperInvariant()
"$zipHash  $(Split-Path $zip -Leaf)" | Set-Content -NoNewline "$zip.sha256"
Write-Output "Bundle: $zip"
Write-Output "SHA256: $zipHash"
