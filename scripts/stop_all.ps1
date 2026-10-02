param(
    [switch]$PreLaunch = $false
)

$ErrorActionPreference = "SilentlyContinue"

# Detect our process and our caller (parent CMD)
$CurrentPid = $PID
$ParentProc = Get-CimInstance Win32_Process -Filter "ProcessId = $CurrentPid" -ErrorAction SilentlyContinue
$ParentPid = if ($ParentProc) { $ParentProc.ParentProcessId } else { 0 }

# Project root directory
$RootDir = if ($PSScriptRoot) { Split-Path -Parent $PSScriptRoot } else { "d:\VirtualCharactor" }
$EscapedRootDir = [regex]::Escape($RootDir)

# 1. Kill sub-service windows (Backend, Frontend, Voice Engine)
# We do NOT use broad "VirtualCharacter*" because that kills the Launcher window too
try {
    taskkill /F /T /FI "WINDOWTITLE eq VirtualCharacter - Backend*" 2>$null
    taskkill /F /T /FI "WINDOWTITLE eq VirtualCharacter - Frontend*" 2>$null
    taskkill /F /T /FI "WINDOWTITLE eq VirtualCharacter - Yae Miko*" 2>$null
} catch {}

# 2. Terminate all VirtualCharacter processes via CIM query (CommandLine inspection)
# We strictly exclude:
# - Our current PowerShell process ($CurrentPid)
# - Our parent CMD process ($ParentPid)
# - Anything related to stop.bat or stop_all.ps1
# - Anything related to the IDE (language_server, Antigravity, chrome-devtools)
$targets = Get-CimInstance Win32_Process -ErrorAction SilentlyContinue | Where-Object {
    $_.ProcessId -ne $CurrentPid -and
    $_.ProcessId -ne $ParentPid -and
    $_.Name -notmatch 'language_server|Antigravity|chrome-devtools' -and
    $_.CommandLine -notmatch 'stop\.bat|stop_all\.ps1|run\.bat|VirtualCharacter Launcher' -and
    (
        ($_.Name -in @('cmd.exe', 'powershell.exe', 'conhost.exe') -and $_.CommandLine -match 'anime_voice_server|vc-server|vc-web|cargo run') -or
        ($_.Name -eq 'python.exe' -and $_.CommandLine -match 'anime_voice_server|anime_voice_synthesizer') -or
        ($_.Name -eq 'node.exe' -and $_.CommandLine -match 'vc-web|vite') -or
        ($_.Name -eq 'vc-server.exe') -or
        ($_.Name -eq 'cargo.exe' -and $_.CommandLine -match 'vc-server')
    )
}

foreach ($target in $targets) {
    try {
        taskkill /F /T /PID $target.ProcessId 2>$null
    } catch {}
}

# 3. Free ports 3000, 5005, 5173 if anything is still listening
$ports = @(3000, 5005, 5173)
foreach ($port in $ports) {
    $conns = Get-NetTCPConnection -LocalPort $port -ErrorAction SilentlyContinue
    if ($conns) {
        $pids = $conns | Select-Object -ExpandProperty OwningProcess -Unique
        foreach ($p in $pids) {
            if ($p -and $p -ne 0 -and $p -ne $CurrentPid -and $p -ne $ParentPid) {
                taskkill /F /T /PID $p 2>$null
            }
        }
    }
}

# 4. Explicit kill for target binaries
Stop-Process -Name "vc-server" -Force -ErrorAction SilentlyContinue

if (-not $PreLaunch) {
    # Check status and display results
    $p3000 = Get-NetTCPConnection -LocalPort 3000 -ErrorAction SilentlyContinue
    $p5005 = Get-NetTCPConnection -LocalPort 5005 -ErrorAction SilentlyContinue
    $p5173 = Get-NetTCPConnection -LocalPort 5173 -ErrorAction SilentlyContinue

    Write-Host ""
    Write-Host "[*] Ket qua kiem tra dich vu:" -ForegroundColor Cyan
    Write-Host ("  - Port 3000 (Backend API & WS) : " + $(if ($p3000) { "[CON CHAY]" } else { "[DA TAT HOAN TOAN]" })) -ForegroundColor $(if ($p3000) { "Red" } else { "Green" })
    Write-Host ("  - Port 5005 (Yae Miko Voice)  : " + $(if ($p5005) { "[CON CHAY]" } else { "[DA TAT HOAN TOAN]" })) -ForegroundColor $(if ($p5005) { "Red" } else { "Green" })
    Write-Host ("  - Port 5173 (Giao dien Web)   : " + $(if ($p5173) { "[CON CHAY]" } else { "[DA TAT HOAN TOAN]" })) -ForegroundColor $(if ($p5173) { "Red" } else { "Green" })
}
