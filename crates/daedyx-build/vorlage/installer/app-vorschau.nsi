; Vorlage für das Setup einer egui-Vorschau (aus Desk M8 / Spike M8, Tasks T1b; ohne Tauri-Bundler).
; Die App bindet sie mit `!include "${VORLAGE}\app-vorschau.nsi"` ein und setzt vorher ihre Kennungen:
;   PRODUCT  Produktname, zugleich Ordner-, Uninstall- und Verknüpfungsname (z. B. „Daedyx Desk Vorschau“)
;   AUMID    App-Kennung für Benachrichtigungen (z. B. de.daedyx.desk.vorschau)
;   EXE      Programmdatei (z. B. daedyx-desk-vorschau.exe)
;   LICENSES Lizenztexte der Bibliotheken (wird als Lizenzen.txt installiert)
; Dazu vom Bauskript (vorlage/installer/installer-bauen.ps1): VERSION, SRCDIR, ICON, OUT, VORLAGE.
; Bildet nach, was der Daedyx Launcher und der Tauri-Updater erwarten:
;   Launcher:  setup.exe /S /D=<Ordner>          Deinstallation: <Ordner>\uninstall.exe /S _?=<Ordner>
;              app.json {"version","exe"} im Installationsordner, Uninstall-Eintrag unter HKCU (registryKey)
;   Updater:   setup.exe /P /UPDATE [/R /ARGS <Argumente>]   (installMode "passive" der Tauri-App 0.7.0)
; Weitere Schalter: /NS (keine Verknüpfung). Nur pro Benutzer (HKCU, keine Adminrechte).
; Probe: vorlage/installer/installer-probe.ps1 (still in Wegwerf-Ordner, Startprobe, Deinstallation, Restsuche).

!ifndef PRODUCT | AUMID | EXE | LICENSES
  !error "PRODUCT, AUMID, EXE und LICENSES muss die App vor dem Einbinden der Vorlage setzen"
!endif
!addincludedir "${__FILEDIR__}"

Unicode true
ManifestDPIAware true
SetCompressor /SOLID lzma
RequestExecutionLevel user

!ifndef VERSION
  !define VERSION "0.1.0"
!endif
!define REGKEY "${PRODUCT}"
!define PUBLISHER "Daedyx"
!define UNINSTKEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${REGKEY}"
!define AUMIDKEY "Software\Classes\AppUserModelId\${AUMID}"
!define LNK "$SMPROGRAMS\${PRODUCT}.lnk"

Name "${PRODUCT}"
OutFile "${OUT}"
InstallDir "$LOCALAPPDATA\${PRODUCT}"
; Wie die Tauri-Vorlage: bisherigen Ordner merken und wiederverwenden. Ruft der Tauri-Updater dieses Setup ohne /D=
; auf, landet die neue Version im selben Ordner (HKCU\Software\daedyx\<Produkt>, Standardwert).
!define DIRKEY "Software\daedyx\${PRODUCT}"
InstallDirRegKey HKCU "${DIRKEY}" ""
Icon "${ICON}"
UninstallIcon "${ICON}"
VIProductVersion "${VERSION}.0"
VIAddVersionKey /LANG=1031 "ProductName" "${PRODUCT}"
VIAddVersionKey /LANG=1031 "FileVersion" "${VERSION}"
VIAddVersionKey /LANG=1031 "ProductVersion" "${VERSION}"
VIAddVersionKey /LANG=1031 "FileDescription" "Setup ${PRODUCT}"
VIAddVersionKey /LANG=1031 "CompanyName" "${PUBLISHER}"
VIAddVersionKey /LANG=1031 "LegalCopyright" "MIT"

!include LogicLib.nsh
!include FileFunc.nsh
!include WordFunc.nsh
!include MUI2.nsh
!include lnk-aumid.nsh

Var Passive
Var NoShortcuts
Var IsUpdate
Var Restart
Var RelaunchArgs

!define MUI_ICON "${ICON}"
!define MUI_UNICON "${ICON}"
!define MUI_PAGE_CUSTOMFUNCTION_PRE SkipIfPassive
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "German"

; Schalter auslesen. /S (still) und /D= (Ordner) versteht NSIS selbst.
Function .onInit
  ${GetParameters} $R0
  ${GetOptions} $R0 "/P" $R1
  ${IfNot} ${Errors}
    StrCpy $Passive 1
    SetAutoClose true
  ${EndIf}
  ClearErrors
  ${GetOptions} $R0 "/NS" $R1
  ${IfNot} ${Errors}
    StrCpy $NoShortcuts 1
  ${EndIf}
  ClearErrors
  ${GetOptions} $R0 "/UPDATE" $R1
  ${IfNot} ${Errors}
    StrCpy $IsUpdate 1
  ${EndIf}
  ClearErrors
  ${GetOptions} $R0 "/R" $R1
  ${IfNot} ${Errors}
    StrCpy $Restart 1
  ${EndIf}
  ClearErrors
  ; Alles nach "/ARGS " geht beim Neustart an die App (wie beim Tauri-Updater).
  ${WordFind} $R0 "/ARGS " "+1}" $R2
  ${If} $R2 != $R0
    StrCpy $RelaunchArgs $R2
  ${EndIf}
FunctionEnd

Function SkipIfPassive
  ${If} $Passive = 1
    Abort
  ${EndIf}
FunctionEnd

Section "${PRODUCT}" SecMain
  ; Laufende App sauber beenden (Pipe-Wort QUIT, M7b), damit die Datei ersetzt werden kann.
  ${If} ${FileExists} "$INSTDIR\${EXE}"
    nsExec::Exec '"$INSTDIR\${EXE}" --quit-running'
    Pop $0
    Sleep 500
  ${EndIf}
  SetOutPath "$INSTDIR"
  File "${SRCDIR}\${EXE}"
  File "/oname=app.ico" "${ICON}"
  File "/oname=Lizenzen.txt" "${LICENSES}"

  ; Für den Launcher: installierte Version und Startprogramm.
  FileOpen $0 "$INSTDIR\app.json" w
  FileWrite $0 '{"version": "${VERSION}", "exe": "${EXE}"}'
  FileClose $0

  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr HKCU "${DIRKEY}" "" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTKEY}" "DisplayName" "${PRODUCT}"
  WriteRegStr HKCU "${UNINSTKEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${UNINSTKEY}" "DisplayIcon" "$INSTDIR\app.ico"
  WriteRegStr HKCU "${UNINSTKEY}" "Publisher" "${PUBLISHER}"
  WriteRegStr HKCU "${UNINSTKEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTKEY}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr HKCU "${UNINSTKEY}" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'
  WriteRegDWORD HKCU "${UNINSTKEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINSTKEY}" "NoRepair" 1
  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  IntFmt $0 "0x%08X" $0
  WriteRegDWORD HKCU "${UNINSTKEY}" "EstimatedSize" "$0"

  ; App-Kennung für Benachrichtigungen (Anzeigename und Symbol im Info-Center).
  WriteRegStr HKCU "${AUMIDKEY}" "DisplayName" "${PRODUCT}"
  WriteRegStr HKCU "${AUMIDKEY}" "IconUri" "$INSTDIR\app.ico"

  ; Startmenü-Verknüpfung mit derselben App-Kennung. Bei /UPDATE nicht neu anlegen, wenn der Nutzer sie gelöscht hat.
  ${If} $NoShortcuts != 1
    ${If} $IsUpdate != 1
    ${OrIf} ${FileExists} "${LNK}"
      CreateShortcut "${LNK}" "$INSTDIR\${EXE}" "" "$INSTDIR\app.ico" 0
      !insertmacro SetLnkAppUserModelId "${LNK}" "${AUMID}"
    ${EndIf}
  ${EndIf}

  ${If} $Restart = 1
    Exec '"$INSTDIR\${EXE}" $RelaunchArgs'
  ${EndIf}
SectionEnd

Section "Uninstall"
  ; App beenden und ihre Spuren außerhalb des Ordners entfernen (Autostart-Wert, App-Kennung, Benachrichtigungen).
  ${If} ${FileExists} "$INSTDIR\${EXE}"
    nsExec::Exec '"$INSTDIR\${EXE}" --quit-running'
    Pop $0
    Sleep 500
    nsExec::Exec '"$INSTDIR\${EXE}" --cleanup'
    Pop $0
  ${EndIf}
  Delete "$INSTDIR\${EXE}"
  Delete "$INSTDIR\app.ico"
  Delete "$INSTDIR\Lizenzen.txt"
  Delete "$INSTDIR\app.json"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  Delete "${LNK}"
  DeleteRegKey HKCU "${UNINSTKEY}"
  DeleteRegKey HKCU "${AUMIDKEY}"
  DeleteRegKey HKCU "${DIRKEY}"
  ; Nutzerdaten der App (eigener Ordner unter %APPDATA%) werden bewusst NICHT gelöscht.
SectionEnd
