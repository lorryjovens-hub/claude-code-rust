param(
    [Parameter(Position=0)]
    [ValidateSet("major","minor","patch")]
    [string]$Bump = "minor",

    [string]$Message = ""
)

$ErrorActionPreference = "Stop"

$pkgPath = "package.json"
$cargoPath = "src-tauri/Cargo.toml"
$tauriPath = "src-tauri/tauri.conf.json"

$pkg = Get-Content $pkgPath -Raw | ConvertFrom-Json
$currentVersion = $pkg.version
Write-Host "[version] Current version: $currentVersion" -ForegroundColor Cyan

$parts = $currentVersion -split '\.'
$major = [int]$parts[0]
$minor = [int]$parts[1]
$patch = [int]$parts[2]

switch ($Bump) {
    "major" { $major++; $minor = 0; $patch = 0 }
    "minor" { $minor++; $patch = 0 }
    "patch" { $patch++ }
}

$newVersion = "$major.$minor.$patch"
Write-Host "[version] New version: $newVersion ($Bump bump)" -ForegroundColor Green

$pkg.version = $newVersion
$pkg | ConvertTo-Json -Depth 100 | Set-Content $pkgPath -Encoding UTF8

$cargo = Get-Content $cargoPath -Raw
$cargo = $cargo -replace "version = `"$currentVersion`"", "version = `"$newVersion`""
Set-Content $cargoPath $cargo -Encoding UTF8NoBOM

$tauri = Get-Content $tauriPath -Raw | ConvertFrom-Json
$tauri.version = $newVersion
$tauri | ConvertTo-Json -Depth 100 | Set-Content $tauriPath -Encoding UTF8

Write-Host "[version] Updated: package.json, Cargo.toml, tauri.conf.json" -ForegroundColor Green

$commitMsg = if ($Message) { "release: v$newVersion - $Message" } else { "release: v$newVersion" }

Write-Host ""
Write-Host "[version] Ready to commit and tag:" -ForegroundColor Yellow
Write-Host "  git add -A" -ForegroundColor White
Write-Host "  git commit -m `"$commitMsg`"" -ForegroundColor White
Write-Host "  git tag -a v$newVersion -m `"v$newVersion`"" -ForegroundColor White
Write-Host ""
Write-Host "[version] To rollback this version:" -ForegroundColor Yellow
Write-Host "  git tag -d v$newVersion" -ForegroundColor White
Write-Host "  git reset --hard HEAD~1" -ForegroundColor White
