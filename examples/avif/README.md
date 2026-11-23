# AVIF loading plugin example

This example registers an `.avif` graphic loading handler with TVP. It uses `image`'s AVIF decoder with default features disabled, so other image formats and the AVIF encoder are not included. The decoder uses dav1d for AV1 decoding.

## Windows build

Run `build.bat` from this directory. The script builds a 32-bit MSVC plugin and expects these tools to be available:

- Visual Studio with the x86 C/C++ toolchain and `vswhere.exe`
- Rust with the `i686-pc-windows-msvc` target (`rustup target add i686-pc-windows-msvc`)
- Git, Meson, Ninja, and `pkg-config` on `PATH`
- NASM when dav1d's x86 assembly optimizations are enabled

The script clones dav1d from VideoLAN into `dav1d/` on the first run. Later runs fetch and fast-forward that checkout. It then uses `vcvarsall.bat x86` and Meson to build a release, static dav1d library with the static MSVC runtime (`/MT`); dav1d tools and tests are disabled. The build is stored in `dav1d-build/`.

After the dav1d build, the script restores the original environment and points `pkg-config` and `system-deps` at the Meson-generated package files. Cargo then builds the plugin for `i686-pc-windows-msvc` with the static Rust runtime. This flow links the locally built static dav1d and does not require vcpkg.

The plugin DLL is written to `target/i686-pc-windows-msvc/release/avif.dll`. On Windows, `build.rs` embeds file and product version resources using the Cargo package version and Git commit count; the displayed version also includes the short commit hash and a dirty-tree marker when applicable.
