    #!/usr/bin/env sh
set -eu

cargo build --release "$@"
maturin build --release --out target/wheels "$@"

printf 'Wheel written to target/wheels/\n'