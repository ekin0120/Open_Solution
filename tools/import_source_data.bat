@echo off
setlocal EnableExtensions
rem Re-import the catalog data from the locally installed E-Solutions 600 MDB set.
rem Optional: set ESOLUTIONS_MDB_DIR to override the source DatabasesPwd folder.
set "TOOLS=%~dp0"
set "WORK=%TOOLS%..\..\import_work"
set "SRC=%ESOLUTIONS_MDB_DIR%"
if "%SRC%"=="" set "SRC=C:\Program Files (x86)\Honeywell International\ESolutions600\DatabasesPwd"

if not exist "%SRC%\*.mdb" (
  echo Source MDB directory not found: %SRC%
  echo Set ESOLUTIONS_MDB_DIR to the DatabasesPwd folder and run again.
  exit /b 1
)
if not exist "%WORK%\mdb" mkdir "%WORK%\mdb"
copy /Y "%SRC%\*.mdb" "%WORK%\mdb\" >nul
if errorlevel 1 (echo Copy failed & exit /b 1)

echo [1/2] Exporting all MDB tables with 32-bit Jet OLEDB 4.0 ...
powershell -NoProfile -ExecutionPolicy Bypass -File "%TOOLS%export_all.ps1" -Root "%WORK%"
if errorlevel 1 (echo Export failed & exit /b 1)

echo [2/2] Importing into dist\data\open_solutions.db ...
python "%TOOLS%import_mdb.py" --dump "%WORK%\dump2"
if errorlevel 1 (echo Import failed & exit /b 1)

echo Done. Restart the GUI to see the refreshed catalog.
endlocal
