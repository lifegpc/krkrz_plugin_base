@ECHO OFF
SETLOCAL
SET "RUSTFLAGS=-C target-feature=+crt-static"
SET "C_STATIC_CRT=true"
cargo build --release --target i686-pc-windows-msvc %*
ENDLOCAL
