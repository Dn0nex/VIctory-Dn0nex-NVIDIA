@echo off
setlocal EnableExtensions DisableDelayedExpansion
rem VIctory launcher. Point it at YOUR OWN dump of the game:
rem   set VICTORY_GAME_DIR=D:\path\to\your\dump      (the folder that holds game_scarlett_master.exe)
rem or put the dump in a folder named "game" next to this file.
if not defined VICTORY_GAME_DIR set "VICTORY_GAME_DIR=%~dp0game"
if not exist "%VICTORY_GAME_DIR%\game_scarlett_master.exe" (
  echo Set VICTORY_GAME_DIR to your own dump folder containing game_scarlett_master.exe. 1>&2
  echo Example:  set VICTORY_GAME_DIR=D:\MyDump  then run this file again. 1>&2
  pause
  exit /b 2
)
set "runner=%~dp0victory.exe"
if defined VICTORY_RUNNER set "runner=%VICTORY_RUNNER%"
if not exist "%runner%" (
  echo victory.exe was not found next to this file. 1>&2
  pause
  exit /b 2
)
if not defined VICTORY_CACHE_DIR set "VICTORY_CACHE_DIR=%LOCALAPPDATA%\VIctory\cache"
if not defined VICTORY_DISPLAY_MODE set "VICTORY_DISPLAY_MODE=1280x720"
if not defined VICTORY_WINDOW_MAX set "VICTORY_WINDOW_MAX=1280x720"
if not defined VICTORY_TEXTURE_ROOM_PERCENT set "VICTORY_TEXTURE_ROOM_PERCENT=82"
if not defined VICTORY_CARD_ROOM_FOR_BUFFERS set "VICTORY_CARD_ROOM_FOR_BUFFERS=1"
if not defined VICTORY_CARD_ROOM_FOR_TARGETS_MB set "VICTORY_CARD_ROOM_FOR_TARGETS_MB=2048"
if not defined VICTORY_RIPE_BLOCKS set "VICTORY_RIPE_BLOCKS=2"
if not defined VICTORY_LEVELS_MIN_KB set "VICTORY_LEVELS_MIN_KB=512"
if not defined VICTORY_ASYNC_SUBMIT set "VICTORY_ASYNC_SUBMIT=1"
if not defined VICTORY_WIDE_MERGE set "VICTORY_WIDE_MERGE=1"
if not defined VICTORY_WIDE_SPIN set "VICTORY_WIDE_SPIN=0"
if not defined VICTORY_AMBIENT_AIR set "VICTORY_AMBIENT_AIR=1.5"
if not defined VICTORY_TEXTURE_NO_WAIT set "VICTORY_TEXTURE_NO_WAIT=1"
if not defined VICTORY_RELAY set "VICTORY_RELAY=2"
if not defined VICTORY_WIDE_BACK set "VICTORY_WIDE_BACK=1"
if not defined VICTORY_CARD_REHASH_RUNS set "VICTORY_CARD_REHASH_RUNS=1"
if not defined VICTORY_JOIN_COPIES set "VICTORY_JOIN_COPIES=1"
if not defined VICTORY_INDIRECT_WINDOW set "VICTORY_INDIRECT_WINDOW=1"
if not defined VICTORY_RELAY_COPY set "VICTORY_RELAY_COPY=1"
if not defined VICTORY_CARD_STEPS set "VICTORY_CARD_STEPS=2"
if not defined VICTORY_INDIRECT_BATCH set "VICTORY_INDIRECT_BATCH=2"
if not defined VICTORY_MAP_THREADS set "VICTORY_MAP_THREADS=1"
set "cpu_threads=%NUMBER_OF_PROCESSORS%"
if not defined cpu_threads set "cpu_threads=12"
set "ram_gb=16"
for /f "usebackq delims=" %%m in (`powershell -NoProfile -Command "[int]((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory/1GB)" 2^>nul`) do set "ram_gb=%%m"
if %cpu_threads% GEQ 14 (
  if not defined VICTORY_THIN_THREADS set "VICTORY_THIN_THREADS=6"
  if not defined VICTORY_LAND_THREADS set "VICTORY_LAND_THREADS=6"
  if not defined VICTORY_WIDE_THREADS set "VICTORY_WIDE_THREADS=6"
  if not defined VICTORY_VIEW_THREADS set "VICTORY_VIEW_THREADS=6"
  if not defined VICTORY_CARD_BUFFERS_MIN_KB set "VICTORY_CARD_BUFFERS_MIN_KB=64"
  if not defined VICTORY_CARD_BUFFERS_MB set "VICTORY_CARD_BUFFERS_MB=1536"
)
if %ram_gb% GEQ 28 (
  if not defined VICTORY_INPLACE_MB set "VICTORY_INPLACE_MB=4096"
)
if not defined VICTORY_CARD_BUFFERS_MB set "VICTORY_CARD_BUFFERS_MB=1024"
if not defined VICTORY_CARD_WINDOWS_MB set "VICTORY_CARD_WINDOWS_MB=1024"
echo VIctory: %cpu_threads% CPU threads, %ram_gb% GB RAM 1>&2
if not defined VICTORY_CARD_AWAKE set "VICTORY_CARD_AWAKE=0"
if not defined VICTORY_SHOW_AFTER set "VICTORY_SHOW_AFTER=1"
if not defined VICTORY_AMBIENT set "VICTORY_AMBIENT=0.8"
if not defined VICTORY_THIN set "VICTORY_THIN=1"
if not defined VICTORY_JOIN_PARTS set "VICTORY_JOIN_PARTS=8"
if not defined VICTORY_UPLOAD_AHEAD set "VICTORY_UPLOAD_AHEAD=1"
if not defined VICTORY_LEVELS_ON_DEMAND set "VICTORY_LEVELS_ON_DEMAND=1"
if not defined VICTORY_CARD_BUFFERS_MIN_KB set "VICTORY_CARD_BUFFERS_MIN_KB=256"
if not defined VICTORY_PLACED_ORDER set "VICTORY_PLACED_ORDER=1"
if not defined VICTORY_PIPELINE_BOOK set "VICTORY_PIPELINE_BOOK=1"
if not defined VICTORY_LAND_SKIP set "VICTORY_LAND_SKIP=1"
if not defined VICTORY_THIN_THREADS set "VICTORY_THIN_THREADS=4"
if not defined VICTORY_WIDE_THREADS set "VICTORY_WIDE_THREADS=2"
if not defined VICTORY_LAND_THREADS set "VICTORY_LAND_THREADS=3"
if not defined VICTORY_VIEW_THREADS set "VICTORY_VIEW_THREADS=2"
if not defined VICTORY_INPLACE set "VICTORY_INPLACE=1"
if not defined VICTORY_INPLACE_MB set "VICTORY_INPLACE_MB=2048"
if not defined VICTORY_WINDOW_KB set "VICTORY_WINDOW_KB=256"
if not defined VICTORY_WINDOW_TAILS set "VICTORY_WINDOW_TAILS=1"
if not defined VICTORY_LOG if not defined RUST_LOG set "VICTORY_LOG=warn,victory_gpu::host=info,victory_gpu::shadow=info,victory_gpu::copystat=info,victory_hle::runtime=info,victory_gpu::land=info,victory_gpu::descriptor=info,victory_gpu::descriptor::host=warn,victory_gpu::list::thin=info,victory_gpu::queue::behind=info"
if not defined VICTORY_FAST_STATS set "VICTORY_FAST_STATS=0"
if not defined VICTORY_RUN_LOG set "VICTORY_RUN_LOG=%~dp0victory-run.log"
if "%VICTORY_RUN_LOG%"=="0" goto to_console
if exist "%VICTORY_RUN_LOG%" move /y "%VICTORY_RUN_LOG%" "%VICTORY_RUN_LOG%.previous" >nul
echo VIctory: the log goes to %VICTORY_RUN_LOG% 1>&2
"%runner%" run "%VICTORY_GAME_DIR%\game_scarlett_master.exe" --console series-s --execute 2>"%VICTORY_RUN_LOG%"
exit /b %errorlevel%
:to_console
"%runner%" run "%VICTORY_GAME_DIR%\game_scarlett_master.exe" --console series-s --execute
exit /b %errorlevel%
