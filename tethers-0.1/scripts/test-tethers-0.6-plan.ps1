Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot '..')
$HostPath = Join-Path $RepoRoot 'host-rust\target\debug\tethers-reference-host.exe'
$EnginePath = Join-Path $RepoRoot 'engine-ocaml\_build\default\bin\tethers_mcp_main.exe'
$Scenario = Join-Path $RepoRoot 'scenarios\j14-complete-local'
$Manifest = Join-Path $RepoRoot 'protocol\capability-manifests\fixture-ping-standing-allow.json'
$Fixture = Join-Path $RepoRoot 'scripts\tethers-stdio-fixture.ps1'

foreach ($path in @($HostPath, $EnginePath, $Manifest, $Fixture)) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "Required file is missing: $path"
    }
}

$temp = Join-Path ([IO.Path]::GetTempPath()) ('tethers-0.6-plan-' + [guid]::NewGuid().ToString('N'))
$workspace = Join-Path $temp 'workspace'
$hostData = Join-Path $workspace 'host-data'
$manifestDir = Join-Path $workspace 'manifests'
$tetherDir = Join-Path $workspace 'tethers'
$scriptsDir = Join-Path $workspace 'scripts'

try {
    New-Item -ItemType Directory -Force -Path $manifestDir, $tetherDir, $scriptsDir, $hostData | Out-Null
    Copy-Item (Join-Path $Scenario 'tethers\complete.tether') (Join-Path $tetherDir 'complete.tether')
    Copy-Item (Join-Path $Scenario 'input.json') (Join-Path $workspace 'input.json')
    Copy-Item $Manifest (Join-Path $manifestDir 'fixture-ping-standing-allow.json')
    Copy-Item $Fixture (Join-Path $scriptsDir 'tethers-stdio-fixture.ps1')

    $marker = Join-Path $workspace 'provider-methods.txt'
    $template = Get-Content -Raw (Join-Path $Scenario 'runtime.template.json') | ConvertFrom-Json
    $template.providers[0].transport.args[2] = Join-Path $scriptsDir 'tethers-stdio-fixture.ps1'
    $template.providers[0].transport.args[5] = $marker
    $template.providers[0].capabilities[0].manifest_path = 'manifests/fixture-ping-standing-allow.json'
    $runtime = Join-Path $workspace 'runtime.json'
    [IO.File]::WriteAllText($runtime, ($template | ConvertTo-Json -Depth 30), [Text.UTF8Encoding]::new($false))

    $output = @(& $HostPath plan --config $runtime --engine $EnginePath --input (Join-Path $workspace 'input.json') --host-data-root $hostData 2>&1)
    $exitCode = $LASTEXITCODE
    $text = ($output -join "`n").Trim()
    if ($exitCode -ne 0) { throw "plan failed with exit ${exitCode}: $text" }
    $envelope = $text | ConvertFrom-Json
    if ($envelope.schema -ne 'tethers.cli/1' -or $envelope.command -ne 'plan' -or $envelope.status -ne 'completed') {
        throw "unexpected plan envelope: $text"
    }
    if ($envelope.data.schema -ne 'tethers.plan/1') { throw 'unexpected plan schema' }
    if ($envelope.data.execution.performed -ne $false) { throw 'plan performed execution' }
    if ([int]$envelope.data.execution.provider_invocations -ne 0) { throw 'plan invoked a provider' }
    if ([int]$envelope.data.execution.trail_execution_entries -ne 0) { throw 'plan wrote Trail entries' }
    if (Test-Path -LiteralPath $marker) { throw 'plan created a provider marker' }
    if (Test-Path -LiteralPath (Join-Path $workspace 'trail.jsonl')) { throw 'plan created Trail' }

    [ordered]@{
        command = 'plan'
        exit_code = $exitCode
        cli_schema = $envelope.schema
        plan_schema = $envelope.data.schema
        plan_action_count = @($envelope.data.plan.actions).Count
        first_action_id = $envelope.data.plan.actions[0].action_id
        execution_performed = $envelope.data.execution.performed
        provider_invocations = $envelope.data.execution.provider_invocations
        trail_execution_entries = $envelope.data.execution.trail_execution_entries
        provider_marker_exists = $false
        trail_exists = $false
    } | ConvertTo-Json -Compress
}
finally {
    Remove-Item -LiteralPath $temp -Recurse -Force -ErrorAction SilentlyContinue
}
