@echo off
echo ====================================
echo TARS Development Environment Setup
echo ====================================
echo.

REM Set Node.js path
set PATH=C:\Program Files\nodejs;%PATH%

echo Checking Node.js installation...
node --version
npm --version
echo.

echo Installing dependencies...
echo This may take a few minutes...
npm install

if %ERRORLEVEL% EQU 0 (
    echo.
    echo Dependencies installed successfully!
    echo.
    echo Starting TARS development server...
    npm run dev
) else (
    echo.
    echo Error installing dependencies!
    echo Please check the error messages above.
)
