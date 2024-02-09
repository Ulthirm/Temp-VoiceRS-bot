@echo off
echo Running first cargo build command...
cargo build --release --target x86_64-pc-windows-gnu
if %ERRORLEVEL% == 0 (
    echo Success: Cargo build for x86_64-pc-windows-gnu completed.
) else (
    echo Error: Cargo build for x86_64-pc-windows-gnu failed.
)
pause

echo Running second cargo build command...
cargo build --release --target x86_64-pc-windows-msvc
if %ERRORLEVEL% == 0 (
    echo Success: Cargo build for x86_64-pc-windows-msvc completed.
) else (
    echo Error: Cargo build for x86_64-pc-windows-msvc failed.
)
pause
