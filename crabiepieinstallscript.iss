; --------------------------------------------------------------
;  CrabiPie.iss — Minimal installer for standalone Rust binary
; --------------------------------------------------------------

#define MyAppName      "CrabiPie"
#define MyAppVersion   "1.0.0"
#define MyAppPublisher "Himal Poudel"
#define MyAppURL       "https://github.com/himalpoudel334/CrabiPie"
#define MyAppExeName   "CrabiPie.exe"

; Path to your standalone binary
#define MyAppSourceDir "C:\Users\himal\Documents\Projects\rust\CrabiPie\target\release"

[Setup]
AppId={{B9E5F7A1-2C3D-4E5F-9A1B-7C8D9E0F1A2B}}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
OutputDir=Output
OutputBaseFilename=CrabiPie_Setup_v{#MyAppVersion}
Compression=lzma
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=admin
ArchitecturesInstallIn64BitMode=x64

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; ONLY copy the standalone .exe — nothing else
Source: "{#MyAppSourceDir}\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}";                     Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}";               Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#MyAppName}}"; Flags: nowait postinstall skipifsilent