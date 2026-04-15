RUSTC   = rustc

RUSTFLAGS = \
    --edition 2021 \
    --emit=obj \
    --target x86_64-unknown-none \
    -C opt-level=2 \
    -C no-redzone=yes \
    -C relocation-model=static \
    -C code-model=kernel \
    -C panic=abort

MODULES = ahci/ahci.km ext2/ext2.km

.PHONY: all clean

all: $(MODULES)

ahci/ahci.km: ahci/src/lib.rs reki.rs
    $(RUSTC) $(RUSTFLAGS) --crate-name ahci $< -o $@
    @echo "  KM  $@"

ext2/ext2.km: ext2/src/lib.rs reki.rs
    $(RUSTC) $(RUSTFLAGS) --crate-name ext2 $< -o $@
    @echo "  KM  $@"

clean:
    rm -f $(MODULES)
