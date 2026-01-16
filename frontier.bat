:: Copyright (c) 2026 The Frontier Framework Authors
:: SPDX-License-Identifier: Apache-2.0 OR MIT

@echo off
set "MANIFEST=.frontier\Cargo.toml"

if "%~1"=="build" goto :BUILD
if "%~1"=="dev" goto :DEV
if "%~1"=="help" goto :HELP
if "%~1"=="" goto :HELP

goto :CARGO_PASS_THROUGH

:BUILD
echo [Frontier] Starting Build...
cargo run --manifest-path %MANIFEST% --bin manager
exit /b 0

:DEV
echo [Frontier] Development Mode...
set FRONTIER_DEV=true
cargo run --manifest-path %MANIFEST% --bin core
exit /b 0

:CARGO_PASS_THROUGH
cargo %* --manifest-path %MANIFEST%
exit /b %errorlevel%

:HELP
echo.
echo Commands:
echo    .\frontier dev      - Test app with Hot Reload
echo    .\frontier build    - Make a final binary
echo    .\back [command]    - Run command in "app\backend"
echo    .\front [command]   - Run command in "app\frontend"
echo.
exit /b 0