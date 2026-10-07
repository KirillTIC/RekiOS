#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"

BREW_PREFIX="$(brew --prefix)"
QEMU="$BREW_PREFIX/bin/qemu-system-x86_64"
E2FS="$BREW_PREFIX/opt/e2fsprogs/sbin"
FIRMWARE="${REKI_FIRMWARE:-$BREW_PREFIX/share/qemu/edk2-x86_64-code.fd}"

if [[ ! -x "$QEMU" || ! -x "$E2FS/mkfs.ext2" ]]; then
    echo 'Install dependencies first: brew install qemu e2fsprogs' >&2
    exit 1
fi
if [[ ! -f "$FIRMWARE" || ! -f target/uefi.img ]]; then
    echo "Missing firmware ($FIRMWARE) or target/uefi.img" >&2
    exit 1
fi

if [[ ! -f target/uefi-vars.fd ]]; then
    cp "$BREW_PREFIX/share/qemu/edk2-i386-vars.fd" target/uefi-vars.fd
fi

if [[ ! -f target/disk.img ]]; then
    seed_dir="$(mktemp -d "${TMPDIR:-/tmp}/reki-seed.XXXXXX")"
    trap 'rm -rf "$seed_dir"' EXIT
    mkdir "$seed_dir/docs"
    echo 'Hello from RekiOS ext2!' > "$seed_dir/hello.txt"
    echo 'RekiOS documentation' > "$seed_dir/docs/readme.txt"
    "$BREW_PREFIX/bin/qemu-img" create -f raw target/disk.img 64M
    "$E2FS/mkfs.ext2" -F -b 1024 -d "$seed_dir" target/disk.img
    rm -rf "$seed_dir"
    trap - EXIT
fi

exec "$QEMU" \
    -name RekiOS \
    -accel tcg \
    -m 128M \
    -drive if=pflash,format=raw,unit=0,readonly=on,file="$FIRMWARE" \
    -drive if=pflash,format=raw,unit=1,file=target/uefi-vars.fd \
    -drive format=raw,file=target/uefi.img \
    -device ahci,id=ahci \
    -drive id=hd0,file=target/disk.img,if=none,format=raw \
    -device ide-hd,drive=hd0,bus=ahci.0 \
    -serial stdio \
    "$@"
