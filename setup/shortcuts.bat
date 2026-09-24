@echo off
setlocal

set "EXE=%USERPROFILE%\bin\CozyMDT.exe"
set "NAME=CozyMDT"

if not exist "%EXE%" (
    echo %NAME%.exe not found in %USERPROFILE%\bin
    timeout /t 2 /nobreak >nul
    exit /b 1
)

set /p choice="Where do you want to create the shortcut [1 for desktop, 2 for start menu, 3 for taskbar (pinned), 4 for desktop and start menu, 5 for all]?: "

if "%choice%"=="1" goto :desktop
if "%choice%"=="2" goto :startmenu
if "%choice%"=="3" goto :taskbar
if "%choice%"=="4" goto :desktop_start
if "%choice%"=="5" goto :all
echo Invalid choice.
timeout /t 1 /nobreak >nul
exit /b 1

:desktop
echo Creating Desktop shortcut...
powershell -noprofile -command "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('%USERPROFILE%\Desktop\%NAME%.lnk'); $s.TargetPath='%EXE%'; $s.WorkingDirectory='%USERPROFILE%\bin'; $s.Save()"
goto :done

:startmenu
echo Creating Start Menu shortcut...
powershell -noprofile -command "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('%APPDATA%\Microsoft\Windows\Start Menu\Programs\%NAME%.lnk'); $s.TargetPath='%EXE%'; $s.WorkingDirectory='%USERPROFILE%\bin'; $s.Save()"
goto :done

:taskbar
echo Creating Taskbar shortcut...
powershell -noprofile -command "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('%APPDATA%\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar\%NAME%.lnk'); $s.TargetPath='%EXE%'; $s.WorkingDirectory='%USERPROFILE%\bin'; $s.Save()"
goto :done

:desktop_start
echo Creating Desktop and Start Menu shortcuts...
powershell -noprofile -command "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('%USERPROFILE%\Desktop\%NAME%.lnk'); $s.TargetPath='%EXE%'; $s.WorkingDirectory='%USERPROFILE%\bin'; $s.Save()"
powershell -noprofile -command "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('%APPDATA%\Microsoft\Windows\Start Menu\Programs\%NAME%.lnk'); $s.TargetPath='%EXE%'; $s.WorkingDirectory='%USERPROFILE%\bin'; $s.Save()"
goto :done

:all
echo Creating all shortcuts...
powershell -noprofile -command "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('%USERPROFILE%\Desktop\%NAME%.lnk'); $s.TargetPath='%EXE%'; $s.WorkingDirectory='%USERPROFILE%\bin'; $s.Save()"
powershell -noprofile -command "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('%APPDATA%\Microsoft\Windows\Start Menu\Programs\%NAME%.lnk'); $s.TargetPath='%EXE%'; $s.WorkingDirectory='%USERPROFILE%\bin'; $s.Save()"
powershell -noprofile -command "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('%APPDATA%\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar\%NAME%.lnk'); $s.TargetPath='%EXE%'; $s.WorkingDirectory='%USERPROFILE%\bin'; $s.Save()"
goto :done

:done
echo Done.
timeout /t 1 /nobreak >nul
endlocal
