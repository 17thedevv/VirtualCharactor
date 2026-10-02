@echo off
title DUNG HE THONG VIRTUALCHARACTER
cd /d "%~dp0"

echo ======================================================================
echo           VIRTUALCHARACTER - DUNG HOAN TOAN CAC DICH VU
echo ======================================================================
echo.
echo [*] Dang tat toan bo tien trinh va dong moi cua so CMD cua VirtualCharacter...

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\stop_all.ps1"

REM Kiem tra them theo cong de dam bao tuyet doi
for /f "tokens=5" %%a in ('netstat -aon ^| findstr :3000 ^| findstr LISTENING') do taskkill /F /T /PID %%a > nul 2>&1
for /f "tokens=5" %%a in ('netstat -aon ^| findstr :5005 ^| findstr LISTENING') do taskkill /F /T /PID %%a > nul 2>&1
for /f "tokens=5" %%a in ('netstat -aon ^| findstr :5173 ^| findstr LISTENING') do taskkill /F /T /PID %%a > nul 2>&1
taskkill /F /IM vc-server.exe > nul 2>&1

REM Dong luon cua so Launcher (run.bat) neu con mo
taskkill /F /FI "WINDOWTITLE eq VirtualCharacter Launcher" > nul 2>&1

echo.
echo ======================================================================
echo  [OK] Da dung sach se 100%% he thong va dong tat ca cua so CMD!
echo  Cua so nay se tu dong dong sau 1 giay...
echo ======================================================================
ping 127.0.0.1 -n 2 > nul
exit
