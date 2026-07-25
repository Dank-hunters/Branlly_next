!macro NSIS_HOOK_POSTINSTALL
  MessageBox MB_YESNO|MB_ICONQUESTION "Créer un raccourci Branlly Next sur le Bureau ?" IDNO desktop_shortcut_done
  CreateShortCut "$DESKTOP\Branlly Next.lnk" "$INSTDIR\Branlly Next.exe"
  desktop_shortcut_done:
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  Delete "$DESKTOP\Branlly Next.lnk"
!macroend
