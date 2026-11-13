#!/usr/bin/env bash
# Runs the tests of `tools/ci` and `tools/assets` under coverage.py and writes their line coverage
# of the scripts beside them to the Cobertura report $1, which SonarQube imports. coverage.py goes into a virtual
# environment of its own, since the runner's Python takes no packages.
#
#   tools/ci/python-coverage.sh python-coverage.xml
set -euo pipefail
: >"$1"
report=$(realpath "$1")
cd "$(dirname "${BASH_SOURCE[0]}")/../.."

venv=$(mktemp -d)
python3 -m venv "$venv"
"$venv/bin/pip" install --quiet coverage==7.17.0
export COVERAGE_FILE="$venv/.coverage"
for folder in tools/ci tools/assets; do
    "$venv/bin/coverage" run --append --source="$folder" --omit="$folder/test_*.py" \
        -m unittest discover -s "$folder"
done
"$venv/bin/coverage" xml -o "$report"
