; ADB can outlive the window and keep AdbWinUsbApi.dll mapped. Register the
; installed executable with Windows Restart Manager, using Tauri's path-scoped
; helper. Never use taskkill /IM adb.exe or adb kill-server: those can stop a
; different installation's Android tools.
;
; GUI initialization runs before Tauri offers to uninstall an older version.
; The section hooks cover silent/update installs and future uninstallers.
!define MUI_CUSTOMFUNCTION_GUIINIT CompanionBeforeUpgrade

Function CompanionBeforeUpgrade
  ; This callback is compiled before Tauri declares its custom variables, so
  ; use only NSIS registers here. The section hooks below use Tauri's helper.
  Push $R0
  Push $0
  Push $1
  Push $2
  Push $3
  !insertmacro RestartManager_StartSession $R0
  ${If} $R0 != ""
    !insertmacro RestartManager_RegisterFile $R0 "$INSTDIR\clean-your-tv-companion.exe"
    !insertmacro RestartManager_RegisterFile $R0 "$INSTDIR\resources\platform-tools\adb.exe"
    System::Call 'RSTRTMGR::RmGetList(p R0, *i .r1, *i .r2, p 0, *i .r3) i .r0'
    ${If} $0 = ${ERROR_MORE_DATA}
      MessageBox MB_OKCANCEL|MB_ICONINFORMATION "Close Clean Your TV and its bundled Android tools to continue? Other Android apps using these tools may disconnect briefly." IDOK companion_close_tools
      !insertmacro RestartManager_EndSession $R0
      Quit
      companion_close_tools:
      System::Call 'RSTRTMGR::RmShutdown(p R0, i ${RmForceShutdown}, p 0) i .r0'
      ${If} $0 != 0
        !insertmacro RestartManager_EndSession $R0
        MessageBox MB_OK|MB_ICONEXCLAMATION "Windows could not close the app's Android tools. Close Clean Your TV and try again. If the file stays in use, restart Windows before running this installer."
        Quit
      ${EndIf}
    ${EndIf}
    !insertmacro RestartManager_EndSession $R0
  ${EndIf}
  Pop $3
  Pop $2
  Pop $1
  Pop $0
  Pop $R0
FunctionEnd

!macro NSIS_HOOK_PREINSTALL
  !insertmacro CheckIfAppIsRunning "$INSTDIR\clean-your-tv-companion.exe" "Clean Your TV"
  !insertmacro CheckIfAppIsRunning "$INSTDIR\resources\platform-tools\adb.exe" "Clean Your TV Android tools"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro CheckIfAppIsRunning "$INSTDIR\clean-your-tv-companion.exe" "Clean Your TV"
  !insertmacro CheckIfAppIsRunning "$INSTDIR\resources\platform-tools\adb.exe" "Clean Your TV Android tools"
!macroend
