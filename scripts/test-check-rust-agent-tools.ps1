[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = $PSScriptRoot | Split-Path -Parent
$checkerScript = Join-Path $repoRoot "scripts/check-rust-agent-tools.ps1"
$configPath = Join-Path $repoRoot "tools/rust-agent-tools.json"

$passed = 0
$failed = 0

function Invoke-Checker {
    param([string]$RepoPath)
    . $checkerScript
    $result = Invoke-RustAgentToolCheck -RepoRoot $RepoPath
    return $result
}

Write-Host "=== Missing config rejection ==="
$testDir1 = Join-Path $env:TEMP "tethers-agent-test-missing"
Remove-Item -LiteralPath $testDir1 -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $testDir1 -ErrorAction Stop | Out-Null
& git clone $repoRoot $testDir1 2>&1 | Out-Null
Remove-Item -LiteralPath (Join-Path $testDir1 "tools/rust-agent-tools.json") -Force -ErrorAction SilentlyContinue
$ec = Invoke-Checker -RepoPath $testDir1
if ($ec -ne 1) { Write-Host "FAIL: missing config (exit $ec, expected 1)"; $script:failed++ }
else { Write-Host "PASS: missing config rejected"; $script:passed++ }
Remove-Item -LiteralPath $testDir1 -Recurse -Force -ErrorAction SilentlyContinue

Write-Host "=== Malformed JSON rejection ==="
$testDir2 = Join-Path $env:TEMP "tethers-agent-test-malformed"
Remove-Item -LiteralPath $testDir2 -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $testDir2 -ErrorAction Stop | Out-Null
& git clone $repoRoot $testDir2 2>&1 | Out-Null
$toolsDir = Join-Path $testDir2 "tools"
if (-not (Test-Path $toolsDir)) { New-Item -ItemType Directory -Path $toolsDir -Force | Out-Null }
Set-Content -LiteralPath (Join-Path $testDir2 "tools/rust-agent-tools.json") -Value "not json" -Encoding UTF8
$ec = Invoke-Checker -RepoPath $testDir2
if ($ec -ne 1) { Write-Host "FAIL: malformed JSON (exit $ec, expected 1)"; $script:failed++ }
else { Write-Host "PASS: malformed JSON rejected"; $script:passed++ }
Remove-Item -LiteralPath $testDir2 -Recurse -Force -ErrorAction SilentlyContinue

Write-Host "=== Wrong schema rejection ==="
$testDir3 = Join-Path $env:TEMP "tethers-agent-test-wrong-schema"
Remove-Item -LiteralPath $testDir3 -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $testDir3 -ErrorAction Stop | Out-Null
& git clone $repoRoot $testDir3 2>&1 | Out-Null
$toolsDir3 = Join-Path $testDir3 "tools"
if (-not (Test-Path $toolsDir3)) { New-Item -ItemType Directory -Path $toolsDir3 -Force | Out-Null }
Set-Content -LiteralPath (Join-Path $testDir3 "tools/rust-agent-tools.json") -Value '{"schema":99,"cargo_nextest":"0.9.140","cargo_deny":"0.19.7","cargo_machete":"0.9.2","rust_analyzer":"toolchain-component"}' -Encoding UTF8
$ec = Invoke-Checker -RepoPath $testDir3
if ($ec -ne 1) { Write-Host "FAIL: wrong schema (exit $ec, expected 1)"; $script:failed++ }
else { Write-Host "PASS: wrong schema rejected"; $script:passed++ }
Remove-Item -LiteralPath $testDir3 -Recurse -Force -ErrorAction SilentlyContinue

Write-Host "=== Impossible configured version rejection ==="
$testDir4 = Join-Path $env:TEMP "tethers-agent-test-impossible"
Remove-Item -LiteralPath $testDir4 -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $testDir4 -ErrorAction Stop | Out-Null
& git clone $repoRoot $testDir4 2>&1 | Out-Null
$toolsDir4 = Join-Path $testDir4 "tools"
if (-not (Test-Path $toolsDir4)) { New-Item -ItemType Directory -Path $toolsDir4 -Force | Out-Null }
Set-Content -LiteralPath (Join-Path $testDir4 "tools/rust-agent-tools.json") -Value '{"schema":1,"cargo_nextest":"9.9.999","cargo_deny":"0.19.7","cargo_machete":"0.9.2","rust_analyzer":"toolchain-component"}' -Encoding UTF8
$ec = Invoke-Checker -RepoPath $testDir4
if ($ec -ne 1) { Write-Host "FAIL: impossible version (exit $ec, expected 1)"; $script:failed++ }
else { Write-Host "PASS: impossible version rejected"; $script:passed++ }
Remove-Item -LiteralPath $testDir4 -Recurse -Force -ErrorAction SilentlyContinue

Write-Host "=== Real accepted configuration ==="
$testDir5 = Join-Path $env:TEMP "tethers-agent-test-real"
Remove-Item -LiteralPath $testDir5 -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $testDir5 -ErrorAction Stop | Out-Null
& git clone $repoRoot $testDir5 2>&1 | Out-Null
$toolsDir5 = Join-Path $testDir5 "tools"
if (-not (Test-Path $toolsDir5)) { New-Item -ItemType Directory -Path $toolsDir5 -Force | Out-Null }
Set-Content -LiteralPath (Join-Path $testDir5 "tools/rust-agent-tools.json") -Value '{"schema":1,"cargo_nextest":"0.9.140","cargo_deny":"0.19.7","cargo_machete":"0.9.2","rust_analyzer":"toolchain-component"}' -Encoding UTF8
# Apply M01B changes to the clone
$rtPath = Join-Path $testDir5 "rust-toolchain.toml"
$rtContent = Get-Content $rtPath -Raw
$rtContent = $rtContent -replace 'components = \["rustfmt", "clippy"\]', 'components = ["rustfmt", "clippy", "rust-analyzer"]'
Set-Content -LiteralPath $rtPath -Value $rtContent -Encoding UTF8
$ocPath = Join-Path $testDir5 "opencode.json"
$ocContent = Get-Content $ocPath -Raw -Encoding UTF8 | ConvertFrom-Json
$ocContent | Add-Member -MemberType NoteProperty -Name "lsp" -Value $true -Force
$ocContent | Add-Member -MemberType NoteProperty -Name "permission" -Value @{ lsp = "allow" } -Force
$ocContent | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $ocPath -Encoding UTF8
$configDir5 = Join-Path $testDir5 ".config"
if (-not (Test-Path $configDir5)) { New-Item -ItemType Directory -Path $configDir5 -Force | Out-Null }
Set-Content -LiteralPath (Join-Path $testDir5 ".config/nextest.toml") -Value "nextest-version = { required = `"0.9.140`" }`n[profile.default]`nretries = 0`nfail-fast = true" -Encoding UTF8
Copy-Item -LiteralPath $configPath -Destination (Join-Path $testDir5 "deny.toml") -ErrorAction SilentlyContinue
$ec = Invoke-Checker -RepoPath $testDir5
if ($ec -ne 0) { Write-Host "FAIL: real config (exit $ec, expected 0)"; $script:failed++ }
else { Write-Host "PASS: real config passed"; $script:passed++ }
Remove-Item -LiteralPath $testDir5 -Recurse -Force -ErrorAction SilentlyContinue

Write-Host "=== Repository non-mutation ==="
$testDir6 = Join-Path $env:TEMP "tethers-agent-test-nonmut"
Remove-Item -LiteralPath $testDir6 -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $testDir6 -ErrorAction Stop | Out-Null
& git clone $repoRoot $testDir6 2>&1 | Out-Null
$beforeGitStatus = & git -C $testDir6 status --porcelain=v1 2>&1 | Out-String
$ec = Invoke-Checker -RepoPath $testDir6
$afterGitStatus = & git -C $testDir6 status --porcelain=v1 2>&1 | Out-String
if ($beforeGitStatus.Trim() -eq $afterGitStatus.Trim()) {
    Write-Host "PASS: repository non-mutation (git status unchanged)"
    $script:passed++
} else {
    Write-Host "FAIL: repository mutation detected"
    Write-Host "Before: $beforeGitStatus"
    Write-Host "After: $afterGitStatus"
    $script:failed++
}
Remove-Item -LiteralPath $testDir6 -Recurse -Force -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "=== Summary ==="
Write-Host "Passed: $passed"
Write-Host "Failed: $failed"

if ($failed -gt 0) {
    exit 1
} else {
    exit 0
}
