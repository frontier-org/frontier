:: Copyright 2026 The Frontier Framework Authors
::
:: Licensed under the Apache License, Version 2.0 (the "License");
:: you may not use this file except in compliance with the License.
:: You may obtain a copy of the License at
::
::     http://www.apache.org/licenses/LICENSE-2.0
::
:: Unless required by applicable law or agreed to in writing, software
:: distributed under the License is distributed on an "AS IS" BASIS,
:: WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
:: See the License for the specific language governing permissions and
:: limitations under the License.

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