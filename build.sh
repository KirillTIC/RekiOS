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
cargo +nightly build -Z build-std=core,compiler_builtins,alloc -Z build-std-features=compiler-builtins-mem

echo "Creating UEFI disk image..."
HOST=$(rustc -vV | grep host | awk '{print $2}')
PROJDIR=$(pwd)
cd /tmp && cargo +nightly run \
  --manifest-path "$PROJDIR/tools/disk-builder/Cargo.toml" \
  --target-dir "$PROJDIR/tools/disk-builder/target" \
  --target "$HOST" \
  -- "$PROJDIR/$KERNEL" "$PROJDIR/$IMAGE"
cd "$PROJDIR"

echo "Launching QEMU..."
qemu-system-x86_64 \
  -bios /usr/share/ovmf/x64/OVMF.4m.fd \
  -drive format=raw,file="$IMAGE"
