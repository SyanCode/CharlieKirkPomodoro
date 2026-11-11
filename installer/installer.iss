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
Name: "{autodesktop}\CharlieKirkPomodoro"; Filename: "{app}\charliekirkpomodoro.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\charliekirkpomodoro.exe"; Description: "Lancer CharlieKirkPomodoro"; Flags: nowait postinstall skipifsilent
