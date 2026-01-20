#!/bin/bash
cargo +nightly build -Z build-std=core,compiler_builtins --target x86_64-reki_os.json
