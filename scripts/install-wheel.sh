#!/usr/bin/env sh
set -eu

wheel=$(
    find target/wheels -maxdepth 1 -type f -name '*.whl' -printf '%T@ %p\n' \
        | sort -nr \
        | sed -n '1s/^[^ ]* //p'
)

if [ -z "$wheel" ]; then
    printf '%s\n' 'No wheel found in target/wheels. Run ./scripts/build-wheel.sh first.' >&2
    exit 1
fi

printf 'Installing %s\n' "$wheel"
python3 -m pip install --force-reinstall "$wheel"