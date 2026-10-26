@echo off
setlocal
cd /d "%~dp0"
if "%~1"=="" (
  echo Usage: install-windows.cmd "C:\path\to\Owlboy"
  exit /b 1
)
where dotnet >nul 2>nul
if errorlevel 1 (echo Install the .NET 9 SDK first. & exit /b 1)
py -3 -m venv .venv
if errorlevel 1 exit /b 1
.venv\Scripts\python.exe -m pip install -r requirements.txt
if errorlevel 1 exit /b 1
.venv\Scripts\python.exe tools\build.py --game "%~1" --out build\package
if errorlevel 1 exit /b 1
.venv\Scripts\python.exe tools\manage.py install --package build\package --game "%~1" --recovery recovery\0.2.0-beta.2
exit /b %errorlevel%
