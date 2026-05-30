param(
    [Parameter(Position=0, Mandatory=$true)]
    [string]$Version,

    [switch]$Force
)

$ErrorActionPreference = "Stop"

$tag = if ($Version.StartsWith("v")) { $Version } else { "v$Version" }

$existingTag = git tag -l $tag 2>$null
if (-not $existingTag) {
    Write-Host "[rollback] Tag $tag not found!" -ForegroundColor Red
    Write-Host "[rollback] Available tags:" -ForegroundColor Yellow
    git tag -l "v*" | ForEach-Object { Write-Host "  $_" -ForegroundColor White }
    exit 1
}

$commitHash = git rev-list -n 1 $tag 2>$null
Write-Host "[rollback] Tag $tag points to commit: $commitHash" -ForegroundColor Cyan

if (-not $Force) {
    Write-Host ""
    Write-Host "[rollback] WARNING: This will reset the repository to $tag" -ForegroundColor Red
    Write-Host "[rollback] Any uncommitted changes will be LOST!" -ForegroundColor Red
    Write-Host "[rollback] Use -Force flag to confirm: .\scripts\rollback.ps1 $Version -Force" -ForegroundColor Yellow
    exit 0
}

git stash 2>$null
git checkout $tag 2>&1 | Write-Host

Write-Host ""
Write-Host "[rollback] Successfully rolled back to $tag" -ForegroundColor Green
Write-Host "[rollback] You are now in detached HEAD state" -ForegroundColor Yellow
Write-Host "[rollback] To create a new branch from here:" -ForegroundColor Yellow
Write-Host "  git checkout -b fix/$Version" -ForegroundColor White
Write-Host "[rollback] To return to main:" -ForegroundColor Yellow
Write-Host "  git checkout main" -ForegroundColor White
