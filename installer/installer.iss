; Installateur Windows (Inno Setup). Compilé par .github/workflows/release.yml
#ifndef AppVersion
  #define AppVersion "1.0.0"
#endif

[Setup]
AppId={{6F1B2C3A-4D5E-4F60-8A7B-9C0D1E2F3A4B}
AppName=CharlieKirkPomodoro
AppVersion={#AppVersion}
AppPublisher=SyanCode
DefaultDirName={autopf}\CharlieKirkPomodoro
DefaultGroupName=CharlieKirkPomodoro
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=Output
OutputBaseFilename=CharlieKirkPomodoro-Setup
Compression=lzma2
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayIcon={app}\charliekirkpomodoro.exe
WizardStyle=modern
SetupIconFile=..\assets\icon.ico
InfoBeforeFile=bienvenue.txt
UninstallDisplayName=CharlieKirkPomodoro

[Languages]
Name: "french"; MessagesFile: "compiler:Languages\French.isl"

[Tasks]
Name: "desktopicon"; Description: "Créer un raccourci sur le Bureau"; GroupDescription: "Raccourcis :"

[Files]
; L'appli + yt-dlp et ffmpeg (nécessaires aux vidéos et à l'audio), tous dans le même dossier.
Source: "..\target\release\charliekirkpomodoro.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "tools\*.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\CharlieKirkPomodoro"; Filename: "{app}\charliekirkpomodoro.exe"
Name: "{group}\Désinstaller CharlieKirkPomodoro"; Filename: "{uninstallexe}"
Name: "{autodesktop}\CharlieKirkPomodoro"; Filename: "{app}\charliekirkpomodoro.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\charliekirkpomodoro.exe"; Description: "Lancer CharlieKirkPomodoro"; Flags: nowait postinstall skipifsilent

[Code]
{ À la désinstallation, propose (sans obliger) de supprimer aussi les memes et réglages. }
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    if MsgBox('L''appli a été désinstallée.' + #13#10 + #13#10 +
              'Veux-tu aussi supprimer tes memes et réglages enregistrés ?' + #13#10 +
              '(Tu peux répondre Non : rien d''autre ne reste sur ton ordinateur.)',
              mbConfirmation, MB_YESNO or MB_DEFBUTTON2) = IDYES then
    begin
      DelTree(ExpandConstant('{userappdata}\SyanCode\CharlieKirkPomodoro'), True, True, True);
      DelTree(ExpandConstant('{localappdata}\SyanCode\CharlieKirkPomodoro'), True, True, True);
    end;
  end;
end;
