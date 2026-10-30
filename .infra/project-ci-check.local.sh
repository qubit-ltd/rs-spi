#!/usr/bin/env bash
set -euo pipefail

project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
package_args=()
case "${SPI_PACKAGE_ALLOW_DIRTY:-0}" in
    0|"") ;;
    1) package_args+=(--allow-dirty) ;;
    *) echo "error: SPI_PACKAGE_ALLOW_DIRTY must be 0 or 1" >&2; exit 2 ;;
esac

cd "$project_root"
python3 -m unittest discover -s scripts/tests -p 'test_*.py'
python3 scripts/check-documentation.py
python3 scripts/check-package.py "${package_args[@]}"
