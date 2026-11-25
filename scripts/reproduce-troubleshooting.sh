#!/usr/bin/env bash
set -euo pipefail

echo "=== DIAGNOSTICO VIctory-CachyOS ==="

echo "--- Kernel ---"
uname -r

echo "--- GPU ---"
lspci | grep -iE 'vga|3d|display' || echo "lspci sin resultado"

echo "--- Driver Vulkan ---"
if command -v vulkaninfo >/dev/null 2>&1; then
  vulkaninfo --summary 2>&1 | grep -E "driverName|apiVersion|deviceName" || true
else
  echo "vulkaninfo no disponible"
fi

echo "--- OOM / GPU reset ---"
if command -v dmesg >/dev/null 2>&1; then
  dmesg -T 2>/dev/null | grep -iE 'oom|amdgpu.*reset|gpu.*fault' | tail -10 || echo "sin errores relevantes"
else
  echo "dmesg no disponible"
fi

echo "--- Memoria ---"
free -h

echo "--- Slab (fugas kernel) ---"
if command -v slabtop >/dev/null 2>&1; then
  sudo slabtop -o 2>/dev/null | head -10 || echo "slabtop requiere sudo"
else
  echo "slabtop no disponible"
fi

echo "--- Rust ---"
rustc --version 2>&1 || true
cargo --version 2>&1 || true
echo "--- Toolchain pinneado ---"
cat rust-toolchain.toml 2>/dev/null || echo "sin rust-toolchain.toml"

echo "--- Ultimos commits ---"
git log --oneline -5 2>/dev/null || true

echo "--- Estado git ---"
git status --short 2>/dev/null || true