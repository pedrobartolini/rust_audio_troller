[Setup]
AppName=PlayClient
AppVersion=1.0
DefaultDirName={pf}\Internet Explorer\SIGNUP\PlayClient
OutputBaseFilename=PlayClient
Compression=lzma
DisableWelcomePage=no
SolidCompression=yes
PrivilegesRequired=admin
UsePreviousAppDir=no
DisableDirPage=yes
DisableProgramGroupPage=yes

[Files]
Source: "target/release/play_client.exe"; DestDir: "{app}"; Flags: ignoreversion

[Run]
Filename: "sc.exe"; Parameters: "delete ""PlayClient"""; Flags: runhidden waituntilterminated
Filename: "sc.exe"; Parameters: "create ""PlayClient"" binPath= ""{app}\play_client.exe"" start= auto DisplayName= ""PlayClient"""; Flags: runhidden waituntilterminated
Filename: "sc.exe"; Parameters: "failure ""PlayClient"" reset= 60 actions= restart/5000/restart/5000/restart/5000"; Flags: runhidden waituntilterminated
Filename: "sc.exe"; Parameters: "start ""PlayClient"""; Flags: runhidden waituntilterminated
Filename: "netsh.exe"; Parameters: "advfirewall firewall add rule name=""PlayClient"" dir=out action=allow program=""{app}\play_client.exe"" enable=yes"; Flags: runhidden waituntilterminated


[UninstallRun]
Filename: "sc.exe"; Parameters: "stop ""PlayClient"""; Flags: runhidden waituntilterminated
Filename: "sc.exe"; Parameters: "delete ""PlayClient"""; Flags: runhidden waituntilterminated
Filename: "netsh.exe"; Parameters: "advfirewall firewall delete rule name=""PlayClient"""; Flags: runhidden waituntilterminated