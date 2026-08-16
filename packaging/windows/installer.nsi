; Oden Windows installer (NSIS).
;
; This is a human-facing installer only ; it is NOT the asset the in-app
; self-updater consumes (that's the plain oden-<target>.zip built alongside
; this in .github/workflows/release.yml). Installs per-user (no admin/UAC
; needed), with Start Menu shortcuts and a proper uninstaller registered in
; "Apps & features".
;
; Built with:
;   makensis /DVERSION=1.2.3 /DARCH=x64 /DSRCDIR=path\to\staging ^
;            /DICONPATH=path\to\icon.ico /DOUTFILE=path\to\Oden-Setup-x64.exe ^
;            packaging\windows\installer.nsi
;
; SRCDIR must contain oden.exe.

!ifndef VERSION
  !define VERSION "0.0.0"
!endif
!ifndef ARCH
  !define ARCH "x64"
!endif
!ifndef SRCDIR
  !define SRCDIR "staging"
!endif
!ifndef ICONPATH
  !define ICONPATH "icon.ico"
!endif
!ifndef OUTFILE
  !define OUTFILE "Oden-Setup-${ARCH}.exe"
!endif

!define APPNAME "Oden"
!define COMPANYNAME "out-of-order"
!define DESCRIPTION "Commands, snippets, and notes, linked together"
!define UNINSTALLKEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\Oden"

!include "MUI2.nsh"

Name "${APPNAME}"
OutFile "${OUTFILE}"
InstallDir "$LOCALAPPDATA\Programs\Oden"
InstallDirRegKey HKCU "Software\Oden" "InstallDir"
RequestExecutionLevel user
Unicode true
VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName" "${APPNAME}"
VIAddVersionKey "CompanyName" "${COMPANYNAME}"
VIAddVersionKey "FileDescription" "${APPNAME} installer"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "ProductVersion" "${VERSION}"
VIAddVersionKey "LegalCopyright" "${COMPANYNAME}"

!define MUI_ABORTWARNING
!define MUI_ICON "${ICONPATH}"
!define MUI_UNICON "${ICONPATH}"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$INSTDIR\oden.exe"
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Section "Oden" SecMain
  SetOutPath "$INSTDIR"
  File "${SRCDIR}\oden.exe"
  File "/oname=icon.ico" "${ICONPATH}"

  WriteRegStr HKCU "Software\Oden" "InstallDir" "$INSTDIR"

  CreateDirectory "$SMPROGRAMS\Oden"
  CreateShortcut "$SMPROGRAMS\Oden\Oden.lnk" "$INSTDIR\oden.exe" "" "$INSTDIR\icon.ico"
  CreateShortcut "$SMPROGRAMS\Oden\Uninstall Oden.lnk" "$INSTDIR\uninstall.exe"

  WriteUninstaller "$INSTDIR\uninstall.exe"

  WriteRegStr HKCU "${UNINSTALLKEY}" "DisplayName" "${APPNAME}"
  WriteRegStr HKCU "${UNINSTALLKEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${UNINSTALLKEY}" "Publisher" "${COMPANYNAME}"
  WriteRegStr HKCU "${UNINSTALLKEY}" "DisplayIcon" "$INSTDIR\icon.ico"
  WriteRegStr HKCU "${UNINSTALLKEY}" "UninstallString" "$INSTDIR\uninstall.exe"
  WriteRegStr HKCU "${UNINSTALLKEY}" "InstallLocation" "$INSTDIR"
  WriteRegDWORD HKCU "${UNINSTALLKEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINSTALLKEY}" "NoRepair" 1
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\oden.exe"
  Delete "$INSTDIR\icon.ico"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"

  Delete "$SMPROGRAMS\Oden\Oden.lnk"
  Delete "$SMPROGRAMS\Oden\Uninstall Oden.lnk"
  RMDir "$SMPROGRAMS\Oden"

  DeleteRegKey HKCU "${UNINSTALLKEY}"
  DeleteRegKey HKCU "Software\Oden"
SectionEnd
