@echo off
setlocal
echo ========================================================
echo   LLMOps Control Center Launcher (Windows)
echo ========================================================
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\start.ps1"
endlocal
