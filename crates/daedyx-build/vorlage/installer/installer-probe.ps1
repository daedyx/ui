# Installationsprobe für ein Setup aus der Vorlage (aus Desk M8, Tasks T1b): still in einen Wegwerf-Ordner
# installieren, Startprobe, Update wie der Tauri-Updater (/P /UPDATE /R /ARGS --hidden) über die laufende App,
# Autostart-Wert setzen, Deinstallation wie der Launcher (/S _?=) und normal (/S), danach alles weg prüfen
# (28 Prüfungen). Nur die übergebenen Kennungen der Vorschau; bricht ab, wenn die Vorschau schon installiert ist
# (fremde Installation wird nie angefasst). Gibt nur Zahlen und OK/FEHLER aus. Rückgabe 0 = alles in Ordnung.
# Die App ruft es über ein eigenes kleines Skript mit ihren Kennungen auf (Desk: scripts/installer-probe-vorschau.ps1).
param(
    [Parameter(Mandatory = $true)][string]$Setup,
    [Parameter(Mandatory = $true)][string]$Product,
    [Parameter(Mandatory = $true)][string]$Exe,
    [Parameter(Mandatory = $true)][string]$Aumid,
    [Parameter(Mandatory = $true)][string]$RunValue,
    [string]$Dir = ""
)
$product = $Product
$exe = $Exe
$processName = [IO.Path]::GetFileNameWithoutExtension($Exe)
if (-not $Dir) { $Dir = Join-Path $env:TEMP "$processName-probe-$PID" }
$uninstKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\$product"
$dirKey = "HKCU:\Software\daedyx\$product"
$aumidKey = "HKCU:\Software\Classes\AppUserModelId\$Aumid"
$runKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
$runValue = $RunValue
$lnk = Join-Path ([Environment]::GetFolderPath("Programs")) "$product.lnk"
$script:failed = 0

function Check($name, $ok) {
    if ($ok) { "OK      $name" } else { "FEHLER  $name"; $script:failed++ }
}
function Run($file, $argLine, $ms = 60000) {
    $p = Start-Process $file -ArgumentList $argLine -PassThru
    if ($p.WaitForExit($ms)) { return $p.ExitCode }
    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
    return "haengt"
}
function Aumid($path) {
    if (-not (Test-Path $path)) { return "" }
    $sh = New-Object -ComObject Shell.Application
    $sh.Namespace((Split-Path $path)).ParseName((Split-Path $path -Leaf)).ExtendedProperty("System.AppUserModel.ID")
}
function RunValue { (Get-ItemProperty $runKey -Name $runValue -ErrorAction SilentlyContinue).$runValue }
function Processes { @(Get-Process $processName -ErrorAction SilentlyContinue | Where-Object { $_.Path -like "$Dir\*" }).Count }
function Gone($tag) {
    Check "$tag Ordner entfernt" (-not (Test-Path $Dir))
    Check "$tag Uninstall-Eintrag entfernt" (-not (Test-Path $uninstKey))
    Check "$tag Ordner-Merkschluessel entfernt" (-not (Test-Path $dirKey))
    Check "$tag App-Kennung entfernt" (-not (Test-Path $aumidKey))
    Check "$tag Verknuepfung entfernt" (-not (Test-Path $lnk))
    Check "$tag Autostart-Wert entfernt" ($null -eq (RunValue))
    Check "$tag keine Prozesse" ((Processes) -eq 0)
}

if (-not (Test-Path $Setup)) { "Setup fehlt: $Setup"; exit 2 }
if ((Test-Path $uninstKey) -or (Test-Path $dirKey) -or ($null -ne (RunValue))) {
    "Abbruch: Die Vorschau ist bereits installiert oder hat einen Autostart-Wert. Die Probe fasst sie nicht an."
    exit 2
}
"Setup KB=" + [math]::Round((Get-Item $Setup).Length / 1KB) + " Ordner=Wegwerf-Ordner unter TEMP"

Check "1 still installieren (/S /D=)" ((Run $Setup "/S /D=$Dir") -eq 0)
$json = if (Test-Path "$Dir\app.json") { Get-Content "$Dir\app.json" -Raw | ConvertFrom-Json } else { $null }
Check "1 app.json mit Version und Startprogramm" ($json -and $json.version -match '^\d+\.\d+\.\d+$' -and $json.exe -eq $exe)
$u = Get-ItemProperty $uninstKey -ErrorAction SilentlyContinue
Check "1 Uninstall-Eintrag mit Ordner und Version" ($u -and $u.InstallLocation -eq $Dir -and $u.DisplayVersion -eq $json.version)
Check "1 Verknuepfung mit App-Kennung" ((Aumid $lnk) -eq $Aumid)
Check "1 App-Kennung in der Registry" (Test-Path $aumidKey)
Check "2 Startprobe des installierten Programms" ((Run "$Dir\$exe" "--start-probe" 15000) -eq 0)

$app = Start-Process "$Dir\$exe" -ArgumentList "--hidden" -PassThru
Start-Sleep -Seconds 3
Check "3 App versteckt gestartet" (-not $app.HasExited)
Check "4 Update wie Tauri-Updater (/P /UPDATE /R /ARGS --hidden, ohne /D=)" ((Run $Setup "/P /UPDATE /R /ARGS --hidden") -eq 0)
Check "4 alte Instanz beendet" ($app.WaitForExit(5000))
Start-Sleep -Seconds 3
Check "4 App neu gestartet, im selben Ordner" ((Processes) -eq 1 -and (Get-ItemProperty $uninstKey).InstallLocation -eq $Dir)

New-ItemProperty $runKey -Name $runValue -Value "`"$Dir\$exe`" --hidden" -Force | Out-Null
Check "5 Autostart-Wert gesetzt (wie der Schalter der App)" ($null -ne (RunValue))
Check "6 Deinstallation wie Launcher (/S _?=)" ((Run "$Dir\uninstall.exe" "/S _?=$Dir") -eq 0)
Start-Sleep -Seconds 2
Remove-Item $Dir -Recurse -Force -ErrorAction SilentlyContinue   # wie der Launcher nach _?=
Gone "6"

Check "7 erneut still installieren" ((Run $Setup "/S /D=$Dir") -eq 0)
Check "7 Deinstallation normal (/S)" ((Run "$Dir\uninstall.exe" "/S") -eq 0)
Start-Sleep -Seconds 3
Gone "7"

if ($script:failed -eq 0) { "ERGEBNIS: alles in Ordnung"; exit 0 } else { "ERGEBNIS: $($script:failed) Fehler"; exit 1 }
