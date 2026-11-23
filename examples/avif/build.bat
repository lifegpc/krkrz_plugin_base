@ECHO OFF
SETLOCAL

REM ---------- Locate Visual Studio ----------
SET "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
IF NOT EXIST "%VSWHERE%" (
    ECHO [ERROR] vswhere.exe not found.
    EXIT /B 1
)

SET "_VS_DIR="
FOR /F "usebackq tokens=*" %%i IN (`"%VSWHERE%" -latest -prerelease -property installationPath`) DO (
    SET "_VS_DIR=%%i"
)
IF NOT DEFINED _VS_DIR (
    ECHO [ERROR] Visual Studio not found.
    EXIT /B 1
)

REM ---------- Paths shared by all stages (set before the inner SETLOCAL) ----------
SET "DAV1D_SRC=%CD%\dav1d"
SET "DAV1D_BUILD=%CD%\dav1d-build"
SET "DAV1D_REPO=https://code.videolan.org/videolan/dav1d.git"

REM ============================================================================
REM Stage 0: fetch / update the dav1d source tree (outside the VS env)
REM ============================================================================
IF NOT EXIST "%DAV1D_SRC%\.git" (
    ECHO [INFO] Cloning dav1d into "%DAV1D_SRC%" ...
    git clone "%DAV1D_REPO%" "%DAV1D_SRC%"
    IF ERRORLEVEL 1 (
        ECHO [ERROR] git clone failed.
        EXIT /B 1
    )
) ELSE (
    ECHO [INFO] Updating dav1d in "%DAV1D_SRC%" ...
    git -C "%DAV1D_SRC%" fetch --all --prune
    IF ERRORLEVEL 1 (
        ECHO [ERROR] git fetch failed.
        EXIT /B 1
    )
    git -C "%DAV1D_SRC%" pull --ff-only
    IF ERRORLEVEL 1 (
        ECHO [ERROR] git pull failed.
        EXIT /B 1
    )
)

REM ============================================================================
REM Stage 1: build dav1d with static CRT (/MT) inside an isolated VS env.
REM The inner SETLOCAL/ENDLOCAL pair guarantees that everything vcvarsall.bat
REM touches (PATH, INCLUDE, LIB, LIBPATH, ...) is rolled back afterwards.
REM ============================================================================
SETLOCAL

CALL "%_VS_DIR%\VC\Auxiliary\Build\vcvarsall.bat" x86
IF ERRORLEVEL 1 (
    ECHO [ERROR] vcvarsall failed.
    ENDLOCAL
    EXIT /B 1
)

REM If the source was updated, wipe the old build dir so Meson reconfigures
REM cleanly and stale /MD object files cannot survive.
IF EXIST "%DAV1D_BUILD%\build.ninja" (
    ECHO [INFO] Reconfiguring existing dav1d build directory ...
    meson setup --reconfigure "%DAV1D_BUILD%" "%DAV1D_SRC%" ^
        --default-library=static ^
        --buildtype=release ^
        -Denable_tools=false ^
        -Denable_tests=false ^
        -Db_vscrt=mt ^
        -Dc_args=/MT ^
        -Dcpp_args=/MT
) ELSE (
    ECHO [INFO] Configuring fresh dav1d build directory ...
    meson setup "%DAV1D_BUILD%" "%DAV1D_SRC%" ^
        --default-library=static ^
        --buildtype=release ^
        -Denable_tools=false ^
        -Denable_tests=false ^
        -Db_vscrt=mt ^
        -Dc_args=/MT ^
        -Dcpp_args=/MT
)
IF ERRORLEVEL 1 (
    ECHO [ERROR] dav1d meson setup failed.
    ENDLOCAL
    EXIT /B 1
)

meson compile -C "%DAV1D_BUILD%"
IF ERRORLEVEL 1 (
    ECHO [ERROR] dav1d meson compile failed.
    ENDLOCAL
    EXIT /B 1
)

ENDLOCAL
REM ----------------------------------------------------------------------------
REM Back to a pristine environment: no VS dirs on PATH, no polluted
REM INCLUDE / LIB, so libclang will not pick up the VS-bundled headers
REM that caused the earlier mmintrin.h errors.
REM ----------------------------------------------------------------------------

REM ============================================================================
REM Stage 2: build the Rust crate against the freshly built dav1d
REM ============================================================================

REM Point system-deps at the Meson-generated uninstalled .pc file.
SET "PKG_CONFIG_PATH=%DAV1D_BUILD%\meson-uninstalled;%DAV1D_BUILD%\meson-private"
SET "PKG_CONFIG_ALLOW_CROSS=1"
SET "SYSTEM_DEPS_DAV1D_SEARCH_NATIVE=%DAV1D_BUILD%\src"

REM Keep the Rust side on the static CRT as well.
SET "RUSTFLAGS=-C target-feature=+crt-static"
SET "C_STATIC_CRT=true"

cargo build --release --target i686-pc-windows-msvc %*
ENDLOCAL