#!/bin/bash
#
#REKI OS
#KIRILL TIC
#CZEPLENOK
#

set -e

KERNEL="target/x86_64-reki_os/debug/reki_os"
IMAGE="target/uefi.img"

echo "Building userspace programs..."
cd programs/calc && cargo +nightly build --release && cd ../..
cd programs/my_first_program && cargo +nightly build --release && cd ../..

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

echo "Creating disk..."
qemu-img create -f raw target/disk.img 64M

echo "Launching QEMU..."
qemu-system-x86_64 \
  -bios /usr/share/ovmf/x64/OVMF.4m.fd \
  -drive format=raw,file="$IMAGE" \
  -device ahci,id=ahci \
  -drive id=hd0,file=target/disk.img,if=none,format=raw \
  -device ide-hd,drive=hd0,bus=ahci.0 \
  -serial stdio
