#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/../.."
DEMO_MANIFEST="$ROOT_DIR/examples/demo/Cargo.toml"

cd "$SCRIPT_DIR"

DIST_DIR="dist/java"
PACKAGE="demo"
BENCH_LIBRARY_BASENAME="bench_uniffi"

export CARGO_TARGET_DIR="$SCRIPT_DIR/target"
export BOLTFFI_DISABLE_EXPORTS=1

cargo build --manifest-path "$DEMO_MANIFEST" --lib --release --features uniffi

rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

if [ "$(uname)" == "Darwin" ]; then
    LIBRARY_FILE="lib${PACKAGE}.dylib"
    BENCH_LIBRARY_FILE="lib${BENCH_LIBRARY_BASENAME}.dylib"
elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
    LIBRARY_FILE="lib${PACKAGE}.so"
    BENCH_LIBRARY_FILE="lib${BENCH_LIBRARY_BASENAME}.so"
else
    echo "Unknown platform: $(uname)"
    exit 1
fi

BINDGEN_JAVA="${UNIFFI_BINDGEN_JAVA:-uniffi-bindgen-java}"

"$BINDGEN_JAVA" generate \
    --out-dir "$DIST_DIR" \
    "target/release/$LIBRARY_FILE"

cp "target/release/$LIBRARY_FILE" "target/release/$BENCH_LIBRARY_FILE"

perl -0pi -e 's/return "demo";/return "bench_uniffi";/g; s/findLibraryName\\("demo"\\)/findLibraryName("bench_uniffi")/g' \
    "$DIST_DIR/uniffi/demo/NamespaceLibrary.java"

echo "Java FFM bindings generated in $DIST_DIR"
