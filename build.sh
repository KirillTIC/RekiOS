#!/bin/bash
#
#REKI OS
#KIRILL TIC
#CZEPLENOK
#

set -e

KERNEL="target/x86_64-reki_os/debug/reki_os"
IMAGE="target/uefi.img"

echo "Building kernel..."
cargo +nightly build -Z build-std=core,compiler_builtins -Z build-std-features=compiler-builtins-mem

echo "Creating UEFI disk image..."
HOST=$(rustc -vV | grep host | awk '{print $2}')
cargo +nightly run --manifest-path tools/disk-builder/Cargo.toml --target "$HOST" -- "$KERNEL" "$IMAGE"

echo "Launching QEMU..."
qemu-system-x86_64 \
  -bios /usr/share/ovmf/x64/OVMF.4m.fd \
  -drive format=raw,file="$IMAGE"
