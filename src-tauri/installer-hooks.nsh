!macro NSIS_HOOK_POSTINSTALL
  nsExec::ExecToStack 'powershell.exe -NoProfile -NonInteractive -File "$INSTDIR\native-host\register-native-host.ps1" -HostExecutable "$INSTDIR\native-host\caixa-preta-native-host.exe"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONEXCLAMATION "O aplicativo foi instalado, mas o registro da ponte falhou. Consulte o README para registrar a ponte localmente."
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::ExecToStack 'powershell.exe -NoProfile -NonInteractive -File "$INSTDIR\native-host\unregister-native-host.ps1" -HostExecutable "$INSTDIR\native-host\caixa-preta-native-host.exe"'
  Pop $0
  Pop $1
!macroend
