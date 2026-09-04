#!/usr/bin/env sh
set -eu

repository=${1:-}

case "$repository" in
    pypi)
        repository_args=""
        ;;
    testpypi)
        repository_args="--repository testpypi"
        ;;
    *)
        printf '%s\n' 'Usage: ./scripts/upload-wheels.sh {testpypi|pypi}' >&2
        exit 2
        ;;
esac

shift
verbose=""
while [ "$#" -gt 0 ]; do
    case "$1" in
        --verbose)
            verbose=--verbose
            ;;
        *)
            printf 'Unknown option: %s\n' "$1" >&2
            printf '%s\n' 'Usage: ./scripts/upload-wheels.sh {testpypi|pypi} [--verbose]' >&2
            exit 2
            ;;
    esac
    shift
done

set -- target/wheels/*.whl target/wheels/*.tar.gz

if [ ! -f "$1" ]; then
    printf '%s\n' 'No package artifacts found. Run ./scripts/build-wheel.sh first.' >&2
    exit 1
fi

twine check "$@"
twine upload $repository_args $verbose "$@"