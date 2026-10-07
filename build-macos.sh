#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"
PROJECT_DIR="$PWD"

# The bootloader's locked dependencies require an older nightly.
BOOTLOADER_TOOLCHAIN=nightly-2025-08-01
rustup target add --toolchain nightly x86_64-unknown-none
rustup component add --toolchain nightly rust-src
if ! rustup run "$BOOTLOADER_TOOLCHAIN" rustc --version >/dev/null 2>&1; then
    rustup toolchain install "$BOOTLOADER_TOOLCHAIN" --profile minimal --component rust-src
fi
make -C kernel_modules -B RUSTC='rustup run nightly rustc'
(cd programs/calc && cargo +nightly build --release)
(cd programs/my_first_program && cargo +nightly build --release)
cargo +nightly build -Z build-std=core,compiler_builtins,alloc -Z build-std-features=compiler-builtins-mem
BUILD_HOST="$(rustc +"$BOOTLOADER_TOOLCHAIN" -vV | awk '/^host:/ {print $2}')"
(
    cd "${TMPDIR:-/tmp}"
    cargo +"$BOOTLOADER_TOOLCHAIN" run \
        --manifest-path "$PROJECT_DIR/tools/disk-builder/Cargo.toml" \
        --target-dir "$PROJECT_DIR/tools/disk-builder/target" \
        --target "$BUILD_HOST" \
        -- "$PROJECT_DIR/target/x86_64-reki_os/debug/reki_os" "$PROJECT_DIR/target/uefi.img"
)
echo 'Build complete. Run ./run-macos.sh to launch RekiOS.'
