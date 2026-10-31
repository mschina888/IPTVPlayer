[Setup]
AppId={{7B1E5C2A-3F4D-4E8A-9C61-IPTVPLAYER0001}
AppName=IPTV Player для Windows 11 x64
AppVersion=0.1.16
AppPublisher=MSChina888
DefaultDirName={autopf}\IPTVPlayer
DefaultGroupName=IPTV Player
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.22000
OutputDir=D:\GIT\Claude\Projects\BuildApps\IPTVPlayer\Release
OutputBaseFilename=IPTVPlayer-Setup-0.1.16
SetupIconFile=D:\GIT\Claude\Projects\BuildApps\IPTVPlayer\Src\assets\app.ico
UninstallDisplayIcon={app}\iptvplayer.exe
Compression=lzma2/ultra64
SolidCompression=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
WizardStyle=modern
; same mutex as the program: setup/uninstall asks to close a running copy
AppMutex=IPTVPlayer_SingleInstance_MSChina888
CloseApplications=yes
RestartApplications=no

[Languages]
Name: "ru"; MessagesFile: "compiler:Languages\Russian.isl"
Name: "en"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"
Name: "autostart"; Description: "Запускать вместе с Windows"; Flags: unchecked

[InstallDelete]
; upgrade: remove everything the previous version put into the program folder (user settings in AppData are kept)
Type: filesandordirs; Name: "{app}\*"
Type: files; Name: "{autodesktop}\IPTV Player.lnk"
Type: filesandordirs; Name: "{group}"

[Files]
Source: "D:\GIT\Claude\Projects\BuildApps\IPTVPlayer\Src\target\release\iptvplayer.exe"; DestDir: "{app}"
Source: "D:\GIT\Claude\Projects\BuildApps\IPTVPlayer\Src\target\release\libmpv-2.dll"; DestDir: "{app}"

[Icons]
Name: "{group}\IPTV Player"; Filename: "{app}\iptvplayer.exe"
Name: "{autodesktop}\IPTV Player"; Filename: "{app}\iptvplayer.exe"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "IPTVPlayer"; ValueData: """{app}\iptvplayer.exe"""; Tasks: autostart

[Run]
Filename: "{app}\iptvplayer.exe"; Description: "Запустить IPTV Player"; Flags: nowait postinstall skipifsilent

[Code]
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    // autostart value may have been written by the program itself
    RegDeleteValue(HKCU, 'Software\Microsoft\Windows\CurrentVersion\Run', 'IPTVPlayer');
    if not UninstallSilent then
      if MsgBox('Удалить также настройки, списки и кэш логотипов IPTV Player?' + #13#10 +
                '(Delete settings, lists and logo cache too?)', mbConfirmation, MB_YESNO or MB_DEFBUTTON2) = IDYES then
      begin
        DelTree(ExpandConstant('{userappdata}\IPTVPlayer'), True, True, True);
        DelTree(ExpandConstant('{localappdata}\IPTVPlayer'), True, True, True);
      end;
  end;
end;