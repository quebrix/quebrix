@echo off
net session >nul 2>&1
if %errorLevel% == 0 (
    echo Running with administrative privileges.
) else (
    echo Requesting administrative privileges.
    powershell -Command "Start-Process cmd -ArgumentList '/c %~fnx0' -Verb RunAs"
    exit
)

:: Get the current path
set CURRENT_PATH=%~dp0

:: Set the path to nssm executable
set NSSM_PATH=%CURRENT_PATH%nssm\nssm.exe

:: Install the Rust executable as a service using nssm
"%NSSM_PATH%" install russel "%CURRENT_PATH%russel.exe" 

:: Start the service (optional)
"%NSSM_PATH%" start russel
