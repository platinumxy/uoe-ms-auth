#!/usr/bin/env sh
set -eu

cargo build --release "$@"
maturin build --release --out target/wheels "$@"
maturin sdist --out target/wheels

printf 'Wheel and source distribution written to target/wheels/\n'