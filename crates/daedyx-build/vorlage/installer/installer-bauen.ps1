# Baut das Setup einer egui-Vorschau aus der Vorlage (app-vorschau.nsi): Release-Bau des Programms, dann makensis.
# Aus Desk M8 (scripts/installer-vorschau.ps1), Tasks T1b. Die App ruft es über ein eigenes kleines Skript mit ihren
# Werten auf (Desk: scripts/installer-vorschau.ps1). Lädt nichts herunter. NSIS wird gesucht: -Makensis, PATH,
# %LOCALAPPDATA%\tauri\NSIS, %ProgramFiles(x86)%\NSIS. Ausgabe: <OutDir>\<Exe ohne .exe>-<Version>-setup.exe,
# dazu Größe und SHA-256.
param(
    [Parameter(Mandatory = $true)][string]$Workspace,
    [Parameter(Mandatory = $true)][string]$Package,
    [Parameter(Mandatory = $true)][string]$Exe,
    [Parameter(Mandatory = $true)][string]$Nsi,
    [Parameter(Mandatory = $true)][string]$Icon,
    [Parameter(Mandatory = $true)][string]$Version,
    [Parameter(Mandatory = $true)][string]$OutDir,
    [string]$Makensis = "",
    [string]$TargetDir = $env:CARGO_TARGET_DIR,
    [switch]$SkipBuild,
    [switch]$Offline
)
$ErrorActionPreference = "Stop"
if (-not $TargetDir) { $TargetDir = Join-Path $Workspace "target" }
if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw "Version muss x.y.z sein: $Version" }

function Find-Makensis {
    if ($Makensis) { return $Makensis }
    $cmd = Get-Command makensis -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    foreach ($p in @("$env:LOCALAPPDATA\tauri\NSIS\makensis.exe", "${env:ProgramFiles(x86)}\NSIS\makensis.exe", "$env:ProgramFiles\NSIS\makensis.exe")) {
        if (Test-Path $p) { return $p }
    }
    throw "makensis nicht gefunden (NSIS 3 installieren oder -Makensis angeben)"
}
$mk = Find-Makensis

if (-not $SkipBuild) {
    $cargoArgs = @("build", "--release", "--locked", "--package", $Package)
    if ($Offline) { $cargoArgs += "--offline" }
    $env:CARGO_TARGET_DIR = $TargetDir
    Push-Location $Workspace
    try { & cargo @cargoArgs; if ($LASTEXITCODE -ne 0) { throw "cargo build fehlgeschlagen ($LASTEXITCODE)" } }
    finally { Pop-Location }
}
$program = Join-Path $TargetDir "release\$Exe"
if (-not (Test-Path $program)) { throw "Programm fehlt: $program" }

$stage = Join-Path $OutDir "app"
New-Item -ItemType Directory -Force $stage | Out-Null
Copy-Item $program $stage -Force
$setup = Join-Path $OutDir ([IO.Path]::GetFileNameWithoutExtension($Exe) + "-$Version-setup.exe")
& $mk /V2 "/DVERSION=$Version" "/DSRCDIR=$stage" "/DICON=$Icon" "/DOUT=$setup" "/DVORLAGE=$PSScriptRoot" $Nsi
if ($LASTEXITCODE -ne 0) { throw "makensis fehlgeschlagen ($LASTEXITCODE)" }

$item = Get-Item $setup
"Setup:   " + $item.FullName
"Version: " + $Version
"Groesse: " + [math]::Round($item.Length / 1KB) + " KB"
"SHA-256: " + (Get-FileHash $setup -Algorithm SHA256).Hash
