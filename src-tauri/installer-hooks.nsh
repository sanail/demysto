; What Windows draws on the taskbar button is not the window's icon: it is the
; picture the shell remembered for this application, and an update in place
; never invalidates it. Somebody who takes a new version over an old one
; therefore keeps the old picture on the button while the tray, the window and
; the file itself all carry the new one — measured on Windows 11, where a
; reboot, a second run of the installer and emptying the icon cache on disk all
; leave it as it was.
;
; Two things together are what moved it, and neither on its own: the shortcut
; has to be written again, and Explorer — which is holding the old picture in
; memory — has to be told that something changed.
;
; Only a shortcut that is already there is written again: the installer
; deliberately creates none while updating, so that one the user threw away
; stays away, and this must not bring it back either.

!macro RefreshShortcutIcon path
  ${If} ${FileExists} "${path}"
    Delete "${path}"
    ; The description is what the shell shows as the shortcut's tooltip. It is
    ; also how a test tells this apart from the installer's own shortcut, which
    ; carries none.
    CreateShortcut "${path}" "$INSTDIR\${MAINBINARYNAME}.exe" "" "" "" "" "" "${PRODUCTNAME}"
    ; The identity the shell groups the windows under. Written again because
    ; the shortcut is a new file, and one without it would leave the taskbar
    ; button no longer belonging to the application.
    !insertmacro SetLnkAppUserModelId "${path}"
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  !if "${STARTMENUFOLDER}" != ""
    !insertmacro RefreshShortcutIcon "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
  !else
    !insertmacro RefreshShortcutIcon "$SMPROGRAMS\${PRODUCTNAME}.lnk"
  !endif
  !insertmacro RefreshShortcutIcon "$DESKTOP\${PRODUCTNAME}.lnk"

  ; And the word to Explorer, which is holding the old picture in memory and
  ; would otherwise go on drawing it until it is restarted.
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, i 0, i 0)'
!macroend
