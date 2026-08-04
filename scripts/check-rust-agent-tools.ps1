[CmdletBinding()]
param(
    [string]$RepoRoot,
    [string]$OpenCodePath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Read-TomlString {
    param([string]$Content, [string]$Key)

    if ($Content -match "${Key}\s*=\s*`"([^`"]+)`"") {
        return $Matches[1]
    }
    return $null
}

function Resolve-OpenCodeApplication {
    param([string]$ExplicitPath)

    $candidate = if ($ExplicitPath) {
        $ExplicitPath
    } elseif ($env:OPENCODE_BIN) {
        $env:OPENCODE_BIN
    } else {
        $command = Get-Command opencode -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($command) { $command.Source } else { $null }
    }

    if (-not $candidate) { return $null }

    try { $item = Get-Item -LiteralPath $candidate -ErrorAction Stop }
    catch { return $null }
    if ($item.PSIsContainer -or $item.Extension.ToLowerInvariant() -notin @('.exe', '.cmd', '.bat', '.com')) { return $null }
    return $item.FullName
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
    if ($Config.rust_analyzer -ne 'toolchain-component') { throw "rust_analyzer must be toolchain-component" }
    foreach ($field in @('cargo_nextest', 'cargo_deny', 'cargo_machete')) {
        if ($Config.$field -isnot [string] -or $Config.$field -notmatch '^\d+\.\d+\.\d+$') {
            throw "$field must be a semantic version such as 0.9.140"
        }
    }
}

function Invoke-RustAgentToolCheck {
    param(
        [string]$RepoRoot,
        [string]$OpenCodePath
    )

    $script:ExitCode = 0
    $script:PassCount = 0
    $script:FailCount = 0

    function Fail {
        param([string]$Message)
        Write-Host "FAIL: $Message"
        $script:FailCount++
        $script:ExitCode = 1
    }

    function Pass {
        param([string]$Message)
        Write-Host "PASS: $Message"
        $script:PassCount++
    }

    if (-not $RepoRoot) {
        $RepoRoot = (git rev-parse --show-toplevel 2>$null).Trim()
    }
    if (-not $RepoRoot -or -not (Test-Path -LiteralPath $RepoRoot -PathType Container)) {
        Fail "repository root not found: $RepoRoot"
        return $script:ExitCode
    }

    $configPath = Join-Path $RepoRoot 'tools/rust-agent-tools.json'
    if (-not (Test-Path -LiteralPath $configPath)) { Fail "config missing: $configPath"; return $script:ExitCode }
    try {
        $config = Get-Content -LiteralPath $configPath -Raw -Encoding UTF8 | ConvertFrom-Json -ErrorAction Stop
        Test-ToolConfiguration -Config $config
        Pass "tool JSON schema and fields are valid"
    } catch {
        Fail "tool JSON invalid: $($_.Exception.Message)"
        return $script:ExitCode
    }

    $rtFile = Join-Path $RepoRoot 'rust-toolchain.toml'
    if (-not (Test-Path -LiteralPath $rtFile)) { Fail 'rust-toolchain.toml missing'; return $script:ExitCode }
    $rtContent = Get-Content -LiteralPath $rtFile -Raw
    $channel = Read-TomlString -Content $rtContent -Key 'channel'
    if (-not $channel) { Fail 'could not read channel from rust-toolchain.toml'; return $script:ExitCode }
    Pass "rust-toolchain.toml channel $channel"
    if ($rtContent -match 'components\s*=\s*\[[^\]]*"rust-analyzer"') { Pass 'rust-analyzer component declared' } else { Fail 'rust-analyzer not declared' }

    $components = @(& rustup component list --toolchain $channel --installed 2>&1 | ForEach-Object { "$_" })
    if ($LASTEXITCODE -ne 0) { Fail 'rustup component list failed' }
    elseif (($components -join "`n") -match 'rust-analyzer') { Pass "rust-analyzer installed for $channel" }
    else { Fail "rust-analyzer not installed for $channel" }

    $raBinary = & rustup which --toolchain $channel rust-analyzer 2>&1
    if ($LASTEXITCODE -ne 0) { Fail "rust-analyzer binary not found: $raBinary" }
    else {
        $raVersion = & $raBinary --version 2>&1 | Select-Object -First 1
        if ($LASTEXITCODE -eq 0) { Pass "rust-analyzer $raVersion" } else { Fail 'rust-analyzer --version failed' }
    }

    foreach ($tool in @(
        @{ Name = 'cargo-nextest'; Expected = $config.cargo_nextest },
        @{ Name = 'cargo-deny'; Expected = $config.cargo_deny },
        @{ Name = 'cargo-machete'; Expected = $config.cargo_machete }
    )) {
        $command = Get-Command $tool.Name -CommandType Application -ErrorAction SilentlyContinue
        if (-not $command) { Fail "$($tool.Name) not found on PATH"; continue }
        $version = & $command.Source --version 2>&1 | Select-Object -First 1
        if ($LASTEXITCODE -ne 0) { Fail "$($tool.Name) --version failed: $version" }
        elseif ($version -match "(^|\s)$([regex]::Escape($tool.Expected))(\s|$)") { Pass "$($tool.Name) $version" }
        else { Fail "$($tool.Name) expected $($tool.Expected), got: $version" }
    }

    $ocPath = Resolve-OpenCodeApplication -ExplicitPath $OpenCodePath
    if (-not $ocPath) {
        Fail 'OpenCode application not found; supply -OpenCodePath, set OPENCODE_BIN, or add opencode to PATH'
    } else {
        $ocVersion = & $ocPath --version 2>&1 | Select-Object -First 1
        if (-not (Test-Path Variable:LASTEXITCODE) -or $LASTEXITCODE -ne 0) { Fail "OpenCode --version failed: $ocVersion" }
        else {
            Pass "OpenCode $ocVersion"
            $debugConfig = & $ocPath debug config 2>&1 | Out-String
            if (-not (Test-Path Variable:LASTEXITCODE) -or $LASTEXITCODE -ne 0) { Fail "OpenCode debug config failed: $debugConfig" }
            elseif ($debugConfig -match '(?s)"lsp"\s*:\s*true' -and $debugConfig -match '(?s)"lsp"\s*:\s*"allow"') { Pass 'OpenCode effective config enables and allows LSP' }
            else { Fail "OpenCode debug config does not prove LSP enabled and allowed: $debugConfig" }
        }
    }

    $ocConfigPath = Join-Path $RepoRoot 'opencode.json'
    try {
        $ocConfig = Get-Content -LiteralPath $ocConfigPath -Raw -Encoding UTF8 | ConvertFrom-Json -ErrorAction Stop
        if ($ocConfig.lsp -eq $true) { Pass 'opencode.json enables LSP' } else { Fail 'opencode.json does not enable LSP' }
        if ($ocConfig.permission.lsp -eq 'allow') { Pass 'opencode.json permits LSP' } else { Fail 'opencode.json does not permit LSP' }
    } catch { Fail "opencode.json invalid: $($_.Exception.Message)" }

    $rootNextest = Join-Path $RepoRoot '.config/nextest.toml'
    $duplicateNextest = Join-Path $RepoRoot 'tethers-0.1/host-rust/.config/nextest.toml'
    if (Test-Path -LiteralPath $rootNextest -PathType Leaf) { Pass 'root .config/nextest.toml present' } else { Fail 'root .config/nextest.toml missing' }
    if (-not (Test-Path -LiteralPath $duplicateNextest)) { Pass 'no duplicate workspace nextest config' } else { Fail 'duplicate workspace nextest config present' }
    if (Test-Path -LiteralPath (Join-Path $RepoRoot 'deny.toml') -PathType Leaf) { Pass 'deny.toml present' } else { Fail 'deny.toml missing' }

    Write-Host "Checks: ${script:PassCount} passed, ${script:FailCount} failed."
    return $script:ExitCode
}

if ($MyInvocation.InvocationName -ne '.') {
    exit (Invoke-RustAgentToolCheck -RepoRoot $RepoRoot -OpenCodePath $OpenCodePath)
}
