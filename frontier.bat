@echo off
set "MANIFEST=.frontier\Cargo.toml"

REM --- ROTEAMENTO ---
if "%~1"=="install" goto :INSTALL
if "%~1"=="build" goto :BUILD
if "%~1"=="dev" goto :DEV
if "%~1"=="help" goto :HELP
if "%~1"=="" goto :HELP

goto :CARGO_PASS_THROUGH

:INSTALL
cargo run --manifest-path %MANIFEST% --bin manager -- %*
exit /b %errorlevel%

:BUILD
echo [Frontier] Iniciando Build...
cargo run --manifest-path %MANIFEST% --bin manager
exit /b 0

:DEV
echo [Frontier] Modo DEV...
set FRONTIER_DEV=true
cargo run --manifest-path %MANIFEST% --bin core
exit /b 0

:CARGO_PASS_THROUGH
cargo %* --manifest-path %MANIFEST%
exit /b %errorlevel%

:HELP
echo.
echo Comandos:
echo   .\frontier dev      - Hot Reload
echo   .\frontier build    - Criar Executavel
echo   .\frontier install  - Instalar Modulo
echo.
exit /b 0