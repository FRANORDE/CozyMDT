@echo off
setlocal

echo Creating executable...
cargo build --release
if errorlevel 1 (
    echo Build failed.
    timeout /t 1 /nobreak >nul
    exit /b 1
)

set "DEST=%USERPROFILE%\bin"
if not exist "%DEST%" mkdir "%DEST%"

echo Copying to %DEST%...
copy /Y "%~dp0target\release\CozyMDT.exe" "%DEST%"
if errorlevel 1 (
    echo Copy failed.
    timeout /t 1 /nobreak >nul
    exit /b 1
)

echo Adding to PATH...
for /f "tokens=2*" %%a in ('reg query "HKCU\Environment" /v Path 2^>nul') do set "CURRENTPATH=%%b"

for %%F in ("%CURRENTPATH%") do if %%~zF GEQ 1000 (
    echo Warning: User PATH is already very long. Add %DEST% manually.
    timeout /t 2 /nobreak >nul
    goto :done
)

echo %CURRENTPATH% | findstr /i "%DEST%" >nul
if errorlevel 1 (
    setx PATH "%CURRENTPATH%;%DEST%"
)

:done
echo Done.
timeout /t 1 /nobreak >nul
endlocal
