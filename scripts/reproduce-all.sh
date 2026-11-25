#!/usr/bin/env bash
set -euo pipefail

echo "=== REPRODUCE ALL: VIctory-CachyOS ==="
echo "Commit: $(git rev-parse --short HEAD)"
echo "Rama: $(git branch --show-current)"
echo "Fecha: $(date -Iseconds)"
echo ""

# Paso 1: build
echo "=== PASO 1: build release ==="
cargo build --release -p victory
echo "OK build"
echo ""

# Paso 2: tests
if [[ "${1:-}" != "--skip-tests" ]]; then
  echo "=== PASO 2: cargo test --workspace ==="
  cargo test --workspace 2>&1 | tail -5
  echo "OK tests"
  echo ""
fi

# Paso 3: gpu-check
echo "=== PASO 3: victory gpu-check ==="
./target/release/victory gpu-check
echo "OK gpu-check"
echo ""

# Paso 4: test-vulkan (requiere display)
if [[ "${1:-}" != "--skip-vulkan" ]]; then
  echo "=== PASO 4: victory test-vulkan ==="
  ./target/release/victory test-vulkan
  echo "OK test-vulkan"
  echo ""
fi

# Paso 5: translate con fixtures (ninguno en el repo)
echo "=== PASO 5: victory translate ==="
SHADER_DIR="tests/fixtures/shaders"
if [[ -d "$SHADER_DIR" ]]; then
  ./target/release/victory translate "$SHADER_DIR" --cache --validate 2>&1 | tail -10
  echo "OK translate"
else
  D=$(mktemp -d)
  ./target/release/victory translate "$D" --cache --validate 2>&1 | grep -E "SPIR-V|validator" | tail -3
  echo "WARN: no hay tests/fixtures/shaders; se usó un directorio vacío (0 shaders)."
fi
echo ""

# Paso 6: run con PE de prueba
echo "=== PASO 6: victory run (PE de prueba) ==="
PE="/tmp/victory-fixture.exe"
if [[ ! -f "$PE" ]]; then
  if command -v clang >/dev/null 2>&1 && command -v lld-link >/dev/null 2>&1; then
    printf 'int main(void){return 0;}\n' > /tmp/victory-fixture.c
    clang --target=x86_64-pc-windows-msvc -c /tmp/victory-fixture.c -o /tmp/victory-fixture.o
    lld-link /subsystem:console /entry:main /out:"$PE" /tmp/victory-fixture.o
  else
    echo "WARN: clang/lld-link no disponibles; no se puede generar PE de prueba."
  fi
fi
if [[ -f "$PE" ]]; then
  ./target/release/victory run "$PE" --execute 2>&1 | tail -12 || true
  echo "WARN: stage2 BLOCKED es esperado (loader Linux no implementado)."
else
  echo "WARN: PE de prueba no disponible; saltando."
fi
echo ""

echo "=== RESUMEN ==="
echo "Ver REPRODUCE.md para estado completo y bloqueos."