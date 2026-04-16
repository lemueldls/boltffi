#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."
ARCHIVE_REPO="$ROOT_DIR/../mobiFFI-benchmarks"
CALLER_DIR="$(pwd)"
INPUT_GLOB="${1:-$ROOT_DIR/benchmarks/*/build/results/**/benchmark_run.json}"

if [[ "$INPUT_GLOB" != /* ]]; then
    INPUT_GLOB="$CALLER_DIR/${INPUT_GLOB#./}"
fi

if [[ ! -d "$ARCHIVE_REPO" ]]; then
    echo "Archive repo not found at $ARCHIVE_REPO" >&2
    exit 1
fi

cd "$ARCHIVE_REPO"
cargo run -- publish --input-glob "$INPUT_GLOB" --site-dir public
npm run build
