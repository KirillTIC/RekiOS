#!/bin/bash
#
#REKI OS
#KIRILL TIC
#CZEPLENOK
#
cargo +nightly build -Z build-std=core,compiler_builtins --target x86_64-reki_os.json
cargo +nightly bootimage
