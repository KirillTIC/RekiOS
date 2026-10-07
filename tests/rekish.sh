#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p target/tests
rustc +nightly --edition 2024 --test tests/rekish.rs -o target/tests/rekish
target/tests/rekish
