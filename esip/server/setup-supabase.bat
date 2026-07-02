@echo off
echo.
echo ========================================
echo DevPlatform - Supabase Setup
echo ========================================
echo.

REM Check if node_modules exists
if not exist "node_modules\" (
  echo Installing dependencies...
  call npm install
  echo.
  echo Dependencies installed!
  echo.
)

REM Generate Prisma Client
echo Generating Prisma Client...
call npm run db:generate
echo.
echo Prisma Client generated!
echo.

REM Push schema to database
echo Pushing database schema to Supabase...
echo WARNING: This will create all tables in your PostgreSQL database.
echo.
set /p confirm="Continue? (y/n): "

if /i "%confirm%"=="y" (
  call npm run db:push
  
  if %errorlevel% equ 0 (
    echo.
    echo ========================================
    echo SUCCESS! Database schema created!
    echo ========================================
    echo.
    echo You can now:
    echo   1. Run 'npm run dev' to start the server
    echo   2. Visit Supabase dashboard at:
    echo      https://supabase.com/dashboard/project/cbotkxhmiwgtepjspitc
    echo   3. Run 'npm run db:studio' for database GUI
    echo.
  ) else (
    echo.
    echo ERROR: Failed to push schema.
    echo Please check your .env file and Supabase connection.
    exit /b 1
  )
) else (
  echo Setup cancelled.
  exit /b 0
)

pause
