#define AppName "CozyMDT"
#define AppVersion "1.2.3"

[Setup]
; Fixed GUID that identifies the app: never change it between versions.
; You can keep this one or generate your own (Inno IDE: Tools > Generate GUID).
AppId={{A3F1C2D4-5B6E-4F70-9A81-2C3D4E5F6A7B}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=FRANORDE
; Same folder init.bat used: %USERPROFILE%\bin
DefaultDirName={%USERPROFILE}\bin
; No admin rights needed, and settings.jsonc can be written next to the exe
PrivilegesRequired=lowest
OutputDir=dist
OutputBaseFilename=CozyMDT-Setup-{#AppVersion}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
UninstallDisplayIcon={app}\CozyMDT.exe
; Close a running CozyMDT during updates, and notify Windows about PATH changes
CloseApplications=yes
ChangesEnvironment=yes

[Files]
Source: "target\release\CozyMDT.exe"; DestDir: "{app}"; Flags: ignoreversion

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; Flags: unchecked
Name: "addtopath"; Description: "Add CozyMDT to the user PATH"

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\CozyMDT.exe"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\CozyMDT.exe"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; \
  ValueData: "{olddata};{app}"; Tasks: addtopath; Check: NeedsAddPath(ExpandConstant('{app}'))

[UninstallDelete]
; settings.jsonc is created at runtime, so the uninstaller doesn't know about it
Type: files; Name: "{app}\settings.jsonc"

[Code]
// Returns True if the folder is not already in the user PATH
function NeedsAddPath(Param: string): Boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', OrigPath) then
  begin
    Result := True;
    exit;
  end;
  Result := Pos(';' + Lowercase(Param) + ';', ';' + Lowercase(OrigPath) + ';') = 0;
end;

// Removes our folder from the user PATH after uninstalling
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  Path, AppDir: string;
  P: Integer;
begin
  if CurUninstallStep <> usPostUninstall then exit;
  if not RegQueryStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', Path) then exit;
  AppDir := ExpandConstant('{app}');
  P := Pos(';' + Lowercase(AppDir), Lowercase(Path));
  if P > 0 then
  begin
    Delete(Path, P, Length(AppDir) + 1);
    RegWriteExpandStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', Path);
  end;
end;
