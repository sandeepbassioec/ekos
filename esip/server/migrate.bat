@echo off
echo.
echo Starting Supabase Migration...
echo.

echo Generating Prisma Client...
call npm run db:generate
echo.

echo Pushing schema to Supabase...
call npm run db:push
echo.

echo ========================================
echo Migration complete!
echo ========================================
echo.
echo Next steps:
echo 1. Start backend: npm run dev
echo 2. Open frontend in browser
echo 3. Open console and run: window.migrateToSupabase()
echo.

pause
