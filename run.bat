@echo off
title VirtualCharacter Launcher
cd /d "%~dp0"

echo ======================================================================
echo           VIRTUALCHARACTER - KHOI DONG HE THONG
echo ======================================================================
echo.

REM 0. Kiem tra cong cu can thiet
where cargo >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Khong tim thay lenh 'cargo' trong PATH.
    echo Hay cai dat Rust tu https://rustup.rs/ va khoi dong lai may.
    echo.
    pause
    exit /b 1
)

where npm >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Khong tim thay lenh 'npm' trong PATH.
    echo Hay cai dat Node.js tu https://nodejs.org/ va khoi dong lai may.
    echo.
    pause
    exit /b 1
)

REM Tu dong don dep sach se cac cua so CMD va tien trinh VirtualCharacter cu (neu co)
echo [*] Dang don dep cac cua so CMD va tien trinh VirtualCharacter cu...
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\stop_all.ps1" -PreLaunch
for /f "tokens=5" %%a in ('netstat -aon ^| findstr :3000 ^| findstr LISTENING') do taskkill /F /T /PID %%a > nul 2>&1
for /f "tokens=5" %%a in ('netstat -aon ^| findstr :5005 ^| findstr LISTENING') do taskkill /F /T /PID %%a > nul 2>&1
for /f "tokens=5" %%a in ('netstat -aon ^| findstr :5173 ^| findstr LISTENING') do taskkill /F /T /PID %%a > nul 2>&1
taskkill /F /IM vc-server.exe > nul 2>&1
echo     [OK] Da don sach cac tien trinh va cua so cu, san sang khoi dong moi.
echo.

REM 1. Kiem tra Ollama
echo [1/4] Dang kiem tra dich vu Ollama Local AI...
curl.exe -s http://127.0.0.1:11434/api/tags > nul 2>&1
if errorlevel 1 (
    echo     [*] Ollama chua chay. Dang khoi dong dich vu Ollama...
    start "Ollama AI Service" /min cmd /c "ollama serve"
    ping 127.0.0.1 -n 4 > nul
    curl.exe -s http://127.0.0.1:11434/api/tags > nul 2>&1
    if errorlevel 1 (
        echo     [*] Chu y: Neu Ollama chua khoi dong, hay mo ung dung Ollama tren may.
    ) else (
        echo     [OK] Da ket noi thanh cong voi Ollama [Model: qwen2.5:3b]
    )
) else (
    echo     [OK] Dich vu Ollama Local AI dang chay san sang.
)
echo.

REM 2. Khoi dong Voice Engine Daemon (Port 5005) - Yae Miko RVC
echo [2/4] Dang kiem tra va khoi dong Yae Miko RVC Voice Engine tai cong 5005...
netstat -ano | findstr :5005 | findstr LISTENING > nul 2>&1
if errorlevel 1 (
    start "VirtualCharacter - Yae Miko Voice Engine (Port 5005)" /min /D "%~dp0" cmd /c "python apps\vc-server\scripts\anime_voice_server.py"
    echo     [OK] Da phat lenh khoi dong Yae Miko RVC Voice Daemon trong nen.
) else (
    echo     [OK] Yae Miko RVC Voice Daemon da dang chay san sang tren cong 5005.
)
echo.

REM 3. Khoi dong Backend (vc-server)
echo [3/4] Dang kiem tra va khoi dong Backend Gateway tai cong 3000...
netstat -ano | findstr :3000 | findstr LISTENING > nul 2>&1
if errorlevel 1 (
    start "VirtualCharacter - Backend (vc-server)" /D "%~dp0" cmd /c "cargo run -p vc-server"
    echo     [OK] Da phat lenh khoi dong Backend Gateway.
) else (
    echo     [OK] Backend Gateway da dang chay tren cong 3000.
)
echo.

REM 4. Kiem tra dependencies va Khoi dong Frontend (vc-web)
echo [4/4] Dang kiem tra va khoi dong Giao dien Web tai cong 5173...
if not exist "apps\vc-web\node_modules\" (
    echo     [*] Lan dau khoi chay: Dang cai dat thu vien npm cho giao dien web...
    cd apps\vc-web && call npm install && cd /d "%~dp0"
)
netstat -ano | findstr :5173 | findstr LISTENING > nul 2>&1
if errorlevel 1 (
    start "VirtualCharacter - Frontend (vc-web)" /D "%~dp0apps\vc-web" cmd /c "npm run dev"
    echo     [OK] Da phat lenh khoi dong Giao dien Web.
) else (
    echo     [OK] Giao dien Web da dang chay tren cong 5173.
)
echo.

REM 5. Cho he thong san sang va mo trinh duyet
echo [*] Dang cho cac dich vu san sang va mo trinh duyet web...
ping 127.0.0.1 -n 4 > nul
start http://localhost:5173/

echo.
echo ======================================================================
echo                    KHOI DONG HOAN TAT
echo ======================================================================
echo  - Giao dien Web UI:       http://localhost:5173/
echo  - Backend API:            http://127.0.0.1:3000/
echo  - Yae Miko Voice Engine:  http://127.0.0.1:5005/
echo.
echo  TINH NANG GIONG NOI VA PHIM TAT:
echo    + Giong noi doc quyen: Onee-san Yae Miko (RVC AI am sac Cao Ty Ty)
echo    + Nut "Thu Giong" o menu: Nghe ngay lap tuc cau chao Yae Miko
echo    + Bieu tuong "Loa" o tin nhan: Phat lai giong noi Onee-san
echo    + Ctrl + H : Mo Trung Tam Lich Su
echo    + ~        : Bat / Tat Mind Inspector
echo    + Esc      : Dong cua so modal
echo.
echo ======================================================================
echo  [CHU Y] De tat sach se tat ca tien trinh va dong moi cua so CMD:
echo         - Chay file stop.bat
echo         - HOAC nhan bat ky phim nao tai day de tu dong tat tat ca va thoat!
echo ======================================================================
echo.
pause
echo.
echo [*] Dang dung sach se toan bo he thong VirtualCharacter...
call "%~dp0stop.bat"
exit
