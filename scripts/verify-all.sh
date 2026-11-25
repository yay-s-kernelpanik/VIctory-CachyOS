#!/usr/bin/env bash
set -u

cd "$(dirname "$0")/.." || exit 1

PASS=0
FAIL=0
RESULTS=()

run_step() {
    local num="$1"
    local desc="$2"
    shift 2
    echo "=== PASO $num: $desc ==="
    if "$@"; then
        echo "  -> OK"
        RESULTS+=("$num  OK   $desc")
        PASS=$((PASS + 1))
    else
        local rc=$?
        echo "  -> FAIL (exit $rc)"
        RESULTS+=("$num  FAIL $desc")
        FAIL=$((FAIL + 1))
    fi
    echo
}

run_step 1 "cargo build --release -p victory" cargo build --release -p victory
run_step 2 "cargo test --workspace" cargo test --workspace
run_step 3 "cargo test -p xtask policy" cargo test -p xtask policy
run_step 4 "victory gpu-check" ./target/release/victory gpu-check
run_step 5 "victory test-vulkan" ./target/release/victory test-vulkan
run_step 6 "victory translate --help" bash -c './target/release/victory translate --help | head -10'

echo "=== RESUMEN ==="
for r in "${RESULTS[@]}"; do
    echo "$r"
done
echo
echo "PASADOS: $PASS  FALLADOS: $FAIL"

if [ "$FAIL" -ne 0 ]; then
    exit 1
fi
exit 0