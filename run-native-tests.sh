#!/usr/bin/env bash
set -euo pipefail
echo "=== Pruebas nativas de VIctory en CachyOS ==="

# Fase 1: Entorno
echo "--- Fase 1: Entorno ---"
rustc --version
cargo --version
echo "LLVM: $(llvm-config --version 2>/dev/null || echo 'N/A')"

# Fase 2: Compilar y testear
echo "--- Fase 2: Compilar ---"
cargo build --release -p victory

echo "--- Fase 2: Tests ---"
cargo test --workspace

# Fase 4: Verificador de shaders
echo "--- Fase 4: Shaders ---"
./target/release/victory translate --help 2>&1 | head -5 || true

# Fase 6: Fixtures
echo "--- Fase 6: Census ---"
./target/release/victory census tests/ 2>&1 | head -10

echo "--- Config ---"
./target/release/victory config

echo "=== Pruebas completadas. Ver docs/NATIVE-LINUX.md ==="
