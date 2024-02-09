@echo off
echo Running Windows cargo build commands...

echo Building for x86_64-pc-windows-gnu...
cargo build --release --target x86_64-pc-windows-gnu
if %ERRORLEVEL% == 0 (
    echo Success: Cargo build for x86_64-pc-windows-gnu completed.
) else (
    echo Error: Cargo build for x86_64-pc-windows-gnu failed.
    pause
    exit /b
)

echo Building for x86_64-pc-windows-msvc...
cargo build --release --target x86_64-pc-windows-msvc
if %ERRORLEVEL% == 0 (
    echo Success: Cargo build for x86_64-pc-windows-msvc completed.
) else (
    echo Error: Cargo build for x86_64-pc-windows-msvc failed.
    pause
    exit /b
)

echo Running Linux cargo build commands via WSL...
wsl bash -c "/mnt/d/VoiceRS/Temp VoiceRS bot/voicers/build_linux.sh"
if %ERRORLEVEL% == 0 (
    echo Success: Linux build commands completed.
) else (
    echo Error: Linux build commands failed.
)

pause
