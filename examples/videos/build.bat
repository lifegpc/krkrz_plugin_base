@ECHO OFF
SETLOCAL
SET "RUSTFLAGS=-C target-feature=+crt-static"
SET "C_STATIC_CRT=true"
REM uncomment follow line to specify xor key, default xor key is 173
REM SET KEY=173
cargo build --release --target i686-pc-windows-msvc %*
ENDLOCAL
