@ECHO OFF
SETLOCAL
SET "RUSTFLAGS=-C target-feature=+crt-static"
SET "C_STATIC_CRT=true"
cargo build --release --target x86_64-pc-windows-msvc %*
ENDLOCAL
