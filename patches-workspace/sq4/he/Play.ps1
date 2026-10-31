param([string]$GamePath, [string]$ScummVM)
$ErrorActionPreference = 'Stop'
if (!$GamePath) {
    $steam = (Get-ItemProperty 'HKCU:\Software\Valve\Steam' -ErrorAction SilentlyContinue).SteamPath
    if ($steam) {
        $libraries = Get-Content (Join-Path $steam 'steamapps\libraryfolders.vdf') |
            ForEach-Object { if ($_ -match '"path"\s+"([^"]+)"') { $Matches[1].Replace('\\','\') } }
        foreach ($library in $libraries) {
            $candidate = Join-Path $library 'steamapps\common\Space Quest Collection\sq4'
            if (Test-Path (Join-Path $candidate 'RESOURCE.MAP')) { $GamePath = $candidate; break }
        }
    }
}
if (!$GamePath) { throw 'Pass the Steam Space Quest IV directory with -GamePath.' }
$map = Join-Path $GamePath 'RESOURCE.MAP'
if ((Get-FileHash -LiteralPath $map).Hash -ne '62384fb1813c1a140072ff84455736c96f28454b3f96a82211ce3dc5844aa4b6') {
    throw 'This patch requires the Steam CD edition of Space Quest IV.'
}
if (!$ScummVM) {
    $ScummVM = Join-Path $env:ProgramFiles 'ScummVM\scummvm.exe'
    if (!(Test-Path $ScummVM)) { $ScummVM = (Get-Command scummvm.exe).Source }
}
$patch = Join-Path $PSScriptRoot 'patch'
if (!(Test-Path (Join-Path $patch '0.msg'))) { throw 'Keep the patch directory next to this launcher.' }
$state = Join-Path $env:LOCALAPPDATA 'Kitor\SQ4-Hebrew'
$null = New-Item -ItemType Directory -Force $state
$config = Join-Path $state 'game.ini'
@"
[scummvm]
updates_check=0
fullscreen=false
aspect_ratio=true
[sq4-he]
engineid=sci
gameid=sq4
platform=dos
language=en
path=$GamePath
extrapath=$patch
subtitles=true
speech_mute=true
"@ | Set-Content -Encoding utf8 $config
Start-Process -FilePath $ScummVM -ArgumentList ('--config="' + $config + '" --logfile="' + (Join-Path $state 'game.log') + '" sq4-he')
