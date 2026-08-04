[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Get-RepoRoot {
    $repo = git rev-parse --show-toplevel 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAIL: git rev-parse failed: $repo"
        exit 1
    }
    $repo = $repo.Trim()
    if (-not (Test-Path -LiteralPath $repo -PathType Container)) {
        Write-Host "FAIL: repository root not found: $repo"
        exit 1
    }
    return $repo
}

function Read-TomlString {
    param([string]$Content, [string]$Key)
    if ($Content -match "$Key\s*=\s*`"([^`"]+)`"") {
        return $Matches[1]
    }
    return $null
}

function Get-CrateName {
    param([string]$SnakeKey)
    return $SnakeKey -replace '_', '-'
}

function Test-ToolConfiguration {
    param($Config)

    $requiredNames = @('schema', 'cargo_nextest', 'cargo_deny', 'cargo_machete', 'rust_analyzer')
    $actualNames = @($Config.PSObject.Properties.Name)
    $unknown = @($actualNames | Where-Object { $_ -notin $requiredNames })
    $missing = @($requiredNames | Where-Object { $_ -notin $actualNames })
    if ($unknown.Count -gt 0) { throw "config has unknown field(s): $($unknown -join ', ')" }
    if ($missing.Count -gt 0) { throw "config missing required field(s): $($missing -join ', ')" }
    if ($Config.schema -isnot [long] -or $Config.schema -ne 1) { throw "expected schema 1, got $($Config.schema)" }
    if ($Config.rust_analyzer -ne 'toolchain-component') { throw 'rust_analyzer must be toolchain-component' }
    foreach ($field in @('cargo_nextest', 'cargo_deny', 'cargo_machete')) {
        if ($Config.$field -isnot [string] -or $Config.$field -notmatch '^\d+\.\d+\.\d+$') {
            throw "$field must be a semantic version such as 0.9.140"
        }
    }
}

$RepoRoot = Get-RepoRoot

$configPath = Join-Path $RepoRoot "tools/rust-agent-tools.json"
if (-not (Test-Path -LiteralPath $configPath)) {
    Write-Host "FAIL: config missing: $configPath"
    exit 1
}

$configText = Get-Content $configPath -Raw -Encoding UTF8
$config = $null
try {
    $config = $configText | ConvertFrom-Json -ErrorAction Stop
    Test-ToolConfiguration -Config $config
} catch {
    Write-Host "FAIL: config is invalid: $($_.Exception.Message)"
    exit 1
}

$rtFile = Join-Path $RepoRoot "rust-toolchain.toml"
if (-not (Test-Path -LiteralPath $rtFile)) {
    Write-Host "FAIL: rust-toolchain.toml missing"
    exit 1
}
$rtContent = Get-Content $rtFile -Raw
$channel = Read-TomlString -Content $rtContent -Key "channel"
if (-not $channel) {
    Write-Host "FAIL: could not read channel from rust-toolchain.toml"
    exit 1
}

$toolchains = @(& rustup toolchain list 2>&1 | ForEach-Object { "$_" })
if ($LASTEXITCODE -ne 0 -or ($toolchains -join "`n") -notmatch [regex]::Escape($channel)) {
    Write-Host "FAIL: Rust toolchain $channel not installed; install before running this script"
    exit 1
}
Write-Host "OK: Rust toolchain $channel is installed"

$components = @(& rustup component list --toolchain $channel --installed 2>&1 | ForEach-Object { "$_" })
$raInstalled = ($components -join "`n") -match "rust-analyzer"
if (-not $raInstalled) {
    Write-Host "INSTALL: adding rust-analyzer component to toolchain $channel"
    & rustup component add rust-analyzer --toolchain $channel 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAIL: rustup component add rust-analyzer failed"
        exit 1
    }
    Write-Host "OK: rust-analyzer added to $channel"
} else {
    Write-Host "OK: rust-analyzer already installed for $channel"
}

$cargoTools = @(
    @{ Name = "cargo-nextest"; Version = $config.cargo_nextest },
    @{ Name = "cargo-deny";    Version = $config.cargo_deny },
    @{ Name = "cargo-machete"; Version = $config.cargo_machete }
)

foreach ($tool in $cargoTools) {
    $crate = Get-CrateName -SnakeKey $tool.Name
    $target = $tool.Version
    $current = $null
    $cmd = Get-Command $tool.Name -CommandType Application -ErrorAction SilentlyContinue
    if ($cmd) {
        $versionOutput = & $tool.Name --version 2>&1 | Select-Object -First 1
        if ($LASTEXITCODE -eq 0) {
            $current = "$versionOutput"
        }
    }

    if ($current -and ($current -match [regex]::Escape($target))) {
        Write-Host "OK: $($tool.Name) $target already installed"
        continue
    }

    $action = if ($current) { "UPDATE" } else { "INSTALL" }
    Write-Host "${action}: $crate $target (current: $($current -replace "`n"," "))"

    $args = @("install", "--locked", "--version", $target, $crate)
    if ($current) {
        $args = @("install", "--locked", "--force", "--version", $target, $crate)
    }

    & cargo @args 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAIL: cargo install $crate $target failed"
        exit 1
    }

    $verifyOutput = & $tool.Name --version 2>&1 | Select-Object -First 1
    if ($LASTEXITCODE -ne 0 -or ($verifyOutput -notmatch [regex]::Escape($target))) {
        Write-Host "FAIL: post-install version check failed for $($tool.Name): $verifyOutput"
        exit 1
    }
    Write-Host "OK: $($tool.Name) $target installed and verified"
}

$raBinary = & rustup which --toolchain $channel rust-analyzer 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "FAIL: rust-analyzer binary not found: $raBinary"
    exit 1
}

$raVer = & $raBinary --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "FAIL: rust-analyzer --version failed"
    exit 1
}
Write-Host "OK: rust-analyzer $raVer"

Write-Host ""
Write-Host "All agent tools installed."
