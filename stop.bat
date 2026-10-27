@echo off
setlocal
echo ========================================================
echo   Stopping LLMOps Control Center Services (Windows)
echo ========================================================
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\stop.ps1"
endlocal
