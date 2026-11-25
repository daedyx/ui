; Setzt an einer Verknüpfung die App-Kennung (System.AppUserModel.ID), damit Windows-Benachrichtigungen der App
; zugeordnet werden. Nur NSIS-Bordmittel: System-Plugin und COM (IShellLinkW, IPersistFile, IPropertyStore).
; Ergebnis in $0: 0 = gespeichert, sonst Fehlercode (HRESULT) oder -1.

!define CLSID_ShellLink "{00021401-0000-0000-C000-000000000046}"
!define IID_IShellLinkW "{000214F9-0000-0000-C000-000000000046}"
!define IID_IPersistFile "{0000010b-0000-0000-C000-000000000046}"
!define IID_IPropertyStore "{886D8EEB-8CF2-4446-8D02-CDBA1DBDCF99}"
; PKEY_AppUserModel_ID = {9F4C2855-9F79-4B39-A8D0-E1D42DE1D5F3}, 5
!define FMTID_AppUserModel "{9F4C2855-9F79-4B39-A8D0-E1D42DE1D5F3}"

!macro SetLnkAppUserModelId LNKPATH APPID
  Push $5
  Push $6
  Push $7
  Push $8
  Push $9
  StrCpy $0 -1
  ; CLSCTX_INPROC_SERVER = 1
  System::Call 'ole32::CoCreateInstance(g "${CLSID_ShellLink}", p 0, i 1, g "${IID_IShellLinkW}", *p .r9) i .r1'
  ${If} $1 = 0
    ; IUnknown::QueryInterface (0) -> IPersistFile
    System::Call '$9->0(g "${IID_IPersistFile}", *p .r8) i .r1'
    ${If} $1 = 0
      ; IPersistFile::Load (5), STGM_READWRITE = 2
      System::Call '$8->5(w "${LNKPATH}", i 2) i .r1'
      ${If} $1 = 0
        System::Call '$9->0(g "${IID_IPropertyStore}", *p .r7) i .r1'
        ${If} $1 = 0
          ; PROPERTYKEY { GUID, DWORD pid = 5 }; PROPVARIANT { VT_LPWSTR = 31, 3x reserviert, Zeiger }
          System::Call '*(&g16 "${FMTID_AppUserModel}", i 5) p .r6'
          System::Call '*(&i2 31, &i2 0, &i2 0, &i2 0, w "${APPID}", i 0) p .r5'
          ; IPropertyStore::SetValue (6), Commit (7)
          System::Call '$7->6(p r6, p r5) i .r1'
          ${If} $1 = 0
            System::Call '$7->7() i .r1'
          ${EndIf}
          ${If} $1 = 0
            ; IPersistFile::Save (6) (NULL, TRUE)
            System::Call '$8->6(p 0, i 1) i .r1'
          ${EndIf}
          StrCpy $0 $1
          System::Free $6
          System::Free $5
          System::Call '$7->2()'
        ${EndIf}
      ${EndIf}
      System::Call '$8->2()'
    ${EndIf}
    System::Call '$9->2()'
  ${EndIf}
  Pop $9
  Pop $8
  Pop $7
  Pop $6
  Pop $5
!macroend
