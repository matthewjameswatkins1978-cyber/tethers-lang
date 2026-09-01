param(
    [switch]$NoFetch,
    [string]$OutputJson
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Invoke-Git {
    param([Parameter(ValueFromRemainingArguments = $true)][string[]]$Arguments)
    $output = & git @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "git $($Arguments -join ' ') failed with exit code $LASTEXITCODE"
    }
    @($output)
}

$root = @(Invoke-Git rev-parse --show-toplevel)[0].Trim()
Push-Location $root
try {
    if (-not $NoFetch) {
        & git fetch origin --prune
        if ($LASTEXITCODE -ne 0) {
            throw "git fetch origin --prune failed with exit code $LASTEXITCODE"
        }
    }

    $main = @(Invoke-Git rev-parse origin/main)[0].Trim()

    $refs = @(
        Invoke-Git for-each-ref `
            "--format=%(refname:short)|%(objectname)|%(committerdate:iso-strict)" `
            refs/remotes/origin
    ) | Where-Object {
        $_ -and
        -not $_.StartsWith("origin/HEAD|") -and
        -not $_.StartsWith("origin/main|")
    }

    $rows = foreach ($line in $refs) {
        $parts = $line -split '\|', 3
        $ref = $parts[0]
        $sha = $parts[1]
        $date = $parts[2]

        $counts = @(Invoke-Git rev-list --left-right --count "origin/main...$ref")[0].Trim()
        $countParts = $counts -split '\s+'
        $behind = [int]$countParts[0]
        $ahead = [int]$countParts[1]

        $classification =
            if ($ahead -eq 0) {
                "ANCESTOR_OF_MAIN"
            }
            elseif ($behind -eq 0) {
                "AHEAD_OF_MAIN_REVIEW"
            }
            else {
                "DIVERGED_UNIQUE_REVIEW"
            }

        [pscustomobject]@{
            branch = $ref.Substring("origin/".Length)
            head = $sha
            ahead_of_main = $ahead
            behind_main = $behind
            last_commit = $date
            classification = $classification
        }
    }

    $rows = @($rows | Sort-Object classification, branch)

    Write-Host ("origin/main: {0}" -f $main)
    Write-Host ("remote branches (excluding main/HEAD): {0}" -f $rows.Count)
    $rows | Format-Table -AutoSize

    if ($OutputJson) {
        $destination = [System.IO.Path]::GetFullPath((Join-Path $root $OutputJson))
        $payload = [pscustomobject]@{
            generated_from_main = $main
            branch_count = $rows.Count
            branches = $rows
        } | ConvertTo-Json -Depth 5
        [System.IO.File]::WriteAllText(
            $destination,
            $payload,
            [System.Text.UTF8Encoding]::new($false)
        )
        Write-Host ("wrote {0}" -f $destination)
    }
}
finally {
    Pop-Location
}
