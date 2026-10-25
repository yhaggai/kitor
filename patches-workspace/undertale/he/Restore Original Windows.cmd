@echo off
cd /d "%~dp0"
if "%~1"=="" (
 echo Drag your backup folder onto this file to restore the original game.
 pause
 exit /b 1
)
py -3 install.py --rollback "%~1"
pause
