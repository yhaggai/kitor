@echo off
cd /d "%~dp0"
py -3 install.py --platform windows %*
if errorlevel 1 echo Installation failed. Check the message above. Python 3.10 or later is required.
pause
