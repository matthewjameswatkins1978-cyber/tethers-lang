[CmdletBinding()]
param(
    [string]$RepoRoot
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Read-TomlString {
    param([string]$Content, [string]$Key)
    if ($Content -match "$Key\s*=\s*`"([^`"]+)`"") {
        return $Matches[1]
    }
    return $null
}

function Invoke-RustAgentToolCheck {
    param(
        [string]$RepoRoot
    )

    $Script:ExitCode = 0
    $Script:CheckOutput = [System.Collections.Generic.List[string]]::new()
    $Script:Errors = [System.Collections.Generic.List[string]]::new()

    function Fail {
        param([string]$Message)
        $msg = "FAIL: $Message"
        Write-Host $msg
        [void]$Script:Errors.Add($msg)
        $Script:ExitCode = 1
    }

    function Pass {
        param([string]$Message)
        $msg = "PASS: $Message"
        Write-Host $msg
        [void]$Script:CheckOutput.Add($msg)
    }

    if (-not $RepoRoot) {
        try { $RepoRoot = git rev-parse --show-toplevel 2>$null }
        catch {}
        if (-not $RepoRoot) { Fail "could not determine repository root"; return $Script:ExitCode }
        $RepoRoot = $RepoRoot.Trim()
    }

    if (-not (Test-Path -LiteralPath $RepoRoot -PathType Container)) {
        Fail "repository root not found: $RepoRoot"
        return $Script:ExitCode
    }

    # Validate config
    $configPath = Join-Path $RepoRoot "tools/rust-agent-tools.json"
    if (-not (Test-Path -LiteralPath $configPath)) {
        Fail "config missing: $configPath"
        return $Script:ExitCode
    }

    $configText = Get-Content $configPath -Raw -Encoding UTF8
    try {
        $config = $configText | ConvertFrom-Json -ErrorAction Stop
    } catch {
        Fail "config is not valid JSON: $_"
        return $Script:ExitCode
    }

    if ($config.schema -ne 1) {
        Fail "expected schema 1, got $($config.schema)"
        return $Script:ExitCode
    }
    Pass "config schema $(1)"

    $expectedConfig = @{
        cargo_nextest   = $config.cargo_nextest
        cargo_deny      = $config.cargo_deny
        cargo_machete   = $config.cargo_machete
        rust_analyzer   = $config.rust_analyzer
    }

    foreach ($key in @("cargo_nextest", "cargo_deny", "cargo_machete", "rust_analyzer")) {
        if (-not $config.$key) {
            Fail "config missing key: $key"
            return $Script:ExitCode
        }
    }

    Pass "config: nextest=$($config.cargo_nextest) deny=$($config.cargo_deny) machete=$($config.cargo_machete) ra=$($config.rust_analyzer)"

    # Verify rust-toolchain.toml
    $rtFile = Join-Path $RepoRoot "rust-toolchain.toml"
    if (-not (Test-Path -LiteralPath $rtFile)) {
        Fail "rust-toolchain.toml missing"
        return $Script:ExitCode
    }
    $rtContent = Get-Content $rtFile -Raw
    $channel = Read-TomlString -Content $rtContent -Key "channel"
    if (-not $channel) {
        Fail "could not read channel from rust-toolchain.toml"
        return $Script:ExitCode
    }
    Pass "rust-toolchain.toml: channel $channel"

    if ($rtContent -match 'components\s*=.*"rust-analyzer"') {
        Pass "rust-toolchain.toml: rust-analyzer component declared"
    } else {
        Fail "rust-toolchain.toml: rust-analyzer not in components"
    }

    # Verify rust-analyzer installed for exact toolchain
    $components = @(& rustup component list --toolchain $channel --installed 2>&1 | ForEach-Object { "$_" })
    if ($LASTEXITCODE -ne 0) {
        Fail "rustup component list failed"
        return $Script:ExitCode
    }

    if (($components -join "`n") -match "rust-analyzer") {
        Pass "rust-analyzer installed for $channel"
    } else {
        Fail "rust-analyzer not installed for $channel"
    }

    # rust-analyzer --version
    $raBinary = & rustup which --toolchain $channel rust-analyzer 2>&1
    if ($LASTEXITCODE -ne 0) {
        Fail "rust-analyzer binary not found: $raBinary"
    } else {
        $raVer = & $raBinary --version 2>&1
        if ($LASTEXITCODE -eq 0) {
            Pass "rust-analyzer: $raVer"
        } else {
            Fail "rust-analyzer --version failed"
        }
    }

    # Cargo tools
    $cargoTools = @(
        @{ Name = "cargo-nextest";  Crate = "cargo-nextest";  Expected = $config.cargo_nextest },
        @{ Name = "cargo-deny";     Crate = "cargo-deny";     Expected = $config.cargo_deny },
        @{ Name = "cargo-machete";  Crate = "cargo-machete";  Expected = $config.cargo_machete }
    )

    foreach ($tool in $cargoTools) {
        $cmd = Get-Command $tool.Name -CommandType Application -ErrorAction SilentlyContinue
        if (-not $cmd) {
            Fail "$($tool.Name) not found on PATH"
            continue
        }
        $versionOutput = & $tool.Name --version 2>&1 | Select-Object -First 1
        if ($LASTEXITCODE -ne 0) {
            Fail "$($tool.Name) --version failed: $versionOutput"
        } elseif ($versionOutput -match [regex]::Escape($tool.Expected)) {
            Pass "$($tool.Name) $versionOutput"
        } else {
            Fail "$($tool.Name) expected $($tool.Expected), got: $versionOutput"
        }
    }

    # OpenCode
    $opencodeCmd = Get-Command opencode -CommandType Application -ErrorAction SilentlyContinue
    if (-not $opencodeCmd) {
        Pass "opencode: not on PATH (wrapper may be required)"
    } else {
        $ocVer = & opencode --version 2>&1 | Select-Object -First 1
        if ($LASTEXITCODE -eq 0) {
            Pass "opencode: $ocVer"
        } else {
            Pass "opencode: present; --version info: $ocVer"
        }
    }

    # OpenCode config check
    $ocConfigPath = Join-Path $RepoRoot "opencode.json"
    if (-not (Test-Path -LiteralPath $ocConfigPath)) {
        Fail "opencode.json missing"
    } else {
        try {
            $ocConfig = Get-Content $ocConfigPath -Raw -Encoding UTF8 | ConvertFrom-Json
        } catch {
            Fail "opencode.json is not valid JSON"
            return $Script:ExitCode
        }
        try {
            if ($ocConfig.lsp -eq $true) { Pass "opencode.json: LSP enabled" }
            else { Fail "opencode.json: LSP not enabled" }
        } catch {
            Fail "opencode.json: LSP not enabled (field missing)"
        }
        try {
            $lspPerm = $ocConfig.permission.lsp
            if ($lspPerm -eq "allow") { Pass "opencode.json: LSP permission allow" }
            else { Fail "opencode.json: LSP permission not allow (got: $lspPerm)" }
        } catch {
            Fail "opencode.json: no permission section"
        }
    }

    # Policy files
    $nextestConfig = Join-Path $RepoRoot ".config/nextest.toml"
    if (Test-Path -LiteralPath $nextestConfig) {
        Pass ".config/nextest.toml present"
    } else {
        Fail ".config/nextest.toml missing"
    }

    $denyConfig = Join-Path $RepoRoot "deny.toml"
    if (Test-Path -LiteralPath $denyConfig) {
        Pass "deny.toml present"
    } else {
        Fail "deny.toml missing"
    }

    Write-Host ""
    $finalMsg = if ($Script:ExitCode -eq 0) { "All agent tool checks passed." } else { "One or more agent tool checks failed." }
    Write-Host $finalMsg
    [void]$Script:CheckOutput.Add($finalMsg)
    return $Script:ExitCode
}

if ($MyInvocation.InvocationName -ne '.') {
    $result = Invoke-RustAgentToolCheck -RepoRoot $RepoRoot
    exit $result
}
