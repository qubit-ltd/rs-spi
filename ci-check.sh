#!/bin/bash
set -euo pipefail

PROJECT_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
python3 -m unittest discover -s "$PROJECT_ROOT/scripts/tests" -p 'test_*.py'
python3 "$PROJECT_ROOT/scripts/check-documentation.py"
package_args=()
if [ "${SPI_PACKAGE_ALLOW_DIRTY:-0}" = "1" ]; then
    package_args+=(--allow-dirty)
fi
python3 "$PROJECT_ROOT/scripts/check-package.py" "${package_args[@]}"
exec env RS_CI_PROJECT_ROOT="$PROJECT_ROOT" "$PROJECT_ROOT/.rs-ci/ci-check.sh" "$@"
