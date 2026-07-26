!macro NSIS_HOOK_POSTINSTALL
  ; Open Spotlight right after install so first-run welcome appears.
  nsis_tauri_utils::RunAsUser "$INSTDIR\${MAINBINARYNAME}.exe" ""
!macroend
