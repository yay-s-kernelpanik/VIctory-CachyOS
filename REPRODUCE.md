# Reproducir VIctory-CachyOS desde cero

Este documento permite a cualquier persona (o a ti en 6 meses) reproducir el
estado actual del port Linux de VIctory, sin ambigüedad, y saber exactamente
qué funciona y qué no.

**Rama:** `feat/vulkan-minimo` · **Plataforma objetivo:** CachyOS / Arch Linux
(x86-64, AMD Radeon con RADV; otras GPUs Vulkan pueden funcionar pero no se
han verificado).

## Requisitos del sistema (CachyOS / Arch)

Instala las dependencias (pacman). Ajusta el driver Vulkan a tu GPU
(`vulkan-radeon` para AMD, `vulkan-intel` para Intel, `nvidia-utils` para
NVIDIA):

```bash
sudo pacman -S --needed \
  base-devel git rustup clang lld \
  vulkan-icd-loader vulkan-tools vulkan-headers \
  spirv-tools glslang \
  mesa vulkan-radeon \
  libxkbcommon wayland
```

Herramientas opcionales pero usadas por el proyecto:

```bash
# SPIR-V validator/disassembler (ya incluido en spirv-tools)
spirv-val --version
spirv-dis --version

# Extractor de RPF9 (no disponible en repos; binario externo `rage-tools-vi`)
# Sin él, translate reporta "RPF9 extraction tool not available" y sigue.
```

Fija el toolchain de Rust según `rust-toolchain.toml`:

```bash
rustup show   # instala el toolchain pinneado automáticamente
rustc --version
cargo --version
```

## 1. Clonar y compilar

```bash
git clone https://github.com/yay-s-kernelpanik/VIctory-CachyOS
cd VIctory-CachyOS
git checkout feat/vulkan-minimo
cargo build --release -p victory
```

Salida esperada (últimas líneas):

```
   Compiling victory v0.1.0 (...)
    Finished `release` profile [optimized] target(s) in ...
```

Si falla: revisa `rust-toolchain.toml` y que `clang`/`lld` estén en `PATH`.

## 2. Verificar tests

```bash
cargo test --workspace
```

Salida esperada:

```
test result: ok. 260 passed; 0 failed; 5 ignored
```

Tests específicos:

```bash
cargo test -p xtask policy          # 4 passed
cargo test -p victory-game rpf9     # 3 passed
cargo test -p victory --test translate_regression   # 4 passed
```

## 3. Verificar GPU (sin datos del juego)

```bash
./target/release/victory gpu-check
```

Salida esperada (final):

```
GPU check: PASS
```

Y en el log:

```
device: AMD Radeon 680M (RADV REMBRANDT) (INTEGRATED_GPU)
vulkan: 1.4.354
GPU check: PASS
```

Si falla: verifica `vulkaninfo --summary`, el driver RADV y que el usuario
esté en el grupo `video`/`render`.

## 4. Verificar Vulkan mínimo (ventana)

```bash
./target/release/victory test-vulkan
```

Salida esperada:

```
Vulkan ready: 800x600, format B8G8R8A8_SRGB
presented one clear-colour frame
```

Se abre una ventana con un color sólido y se cierra tras presentar un frame.
Requiere sesión gráfica (X11/Wayland). En CI sin display, usa
`--skip-vulkan`.

## 5. Traducir shaders (sin datos del juego)

No hay fixtures de shaders en el repo (política de assets). Para verificar
la conversión DXIL→SPIR-V usa un directorio vacío o un contenedor propio:

```bash
D=$(mktemp -d)
./target/release/victory translate "$D"
./target/release/victory translate --cache --validate "$D"
```

Salida esperada:

```
SPIR-V       0 shaders converted (0 graphics, 0 compute, 0 mesh; 0 validated)
validator    SPIRV-Tools v2026.4 ...
```

Con shaders reales (DXBC/DXIL SM4/SM5/SM6) el contador sube y `spirv-val`
valida cada `.spv` (en `--cache`, se escriben en `<cache>/shaders`).

## 6. Ejecutar el cargador con un PE de prueba

No hay PE de prueba en el repo. Genera uno mínimo (Windows PE32+):

```bash
cat > /tmp/victory-fixture.c <<'EOF'
int main(void) { return 0; }
EOF
clang --target=x86_64-pc-windows-msvc -c /tmp/victory-fixture.c -o /tmp/victory-fixture.o
lld-link /subsystem:console /entry:main /out:/tmp/victory-fixture.exe /tmp/victory-fixture.o
```

Ejecuta:

```bash
./target/release/victory run /tmp/victory-fixture.exe --execute
```

Salida esperada:

```
stage 1/3 prepare   ... OK (PE parse/patch, build, imports)
stage 2/3 map/bind  ... BLOCKED (loader Linux not implemented)
stage 3/3 Vulkan    ... ready + presented one clear-colour frame
```

### Por qué stage 2 está bloqueado

`victory_loader::load` y los módulos de `victory-hle` están gateados con
`#[cfg(windows)]`. El port Linux del mapeo/ejecución del PE (relocations,
IAT, heap) no está implementado. Ver `docs/WINDOWS-GATING.md` y
`docs/HLE-INVENTORY.md`.

## 7. Qué NO funciona (y por qué)

| Feature | Estado | Bloqueo | Archivo a tocar |
|---|---|---|---|
| Cargar PE real | ❌ | Loader `cfg(windows)` | `crates/victory-loader/src/lib.rs` |
| Ejecutar código del juego | ❌ | Loader + HLE | `victory-loader` + `victory-hle` |
| Shaders del juego | ❌ | RPF9 no integrado | `crates/victory-game/src/rpf9.rs` |
| Menú del juego | ❌ | Todo lo anterior | — |
| Pipelines gráficos | ❌ | Solo compute implementado | `crates/victory-gpu/src/host_vulkan.rs` |
| HLE (kernel/display/storage) | ❌ | 100% Windows | `crates/victory-hle/src/modules/*` |
| Shaders de geometría (Gs/Ls/Hs/Es) | ❌ | No emitidos | `crates/victory-shader/src/emit/*` |

## 8. Troubleshooting

- **Pantalla azul / GPU reset al probar Vulkan:**
  ```bash
  dmesg -T | grep -iE 'oom|amdgpu.*reset|gpu.*fault' | tail -20
  ```
  Revisa kernel/amdgpu; prueba `amdgpu.dcdebugmask`.

- **Fuga de RAM:**
  ```bash
  free -h
  sudo slabtop -o | head -15
  journalctl -b | grep -i oom | tail
  ```

- **`victory gpu-check` FAIL:**
  ```bash
  vulkaninfo --summary
  echo $VULKAN_ICD_FILENAMES
  ```
  Verifica driver RADV (`vulkan-radeon`) y grupos `video`/`render`.

- **Tests fallan:**
  ```bash
  rm -rf target/
  rustup update
  cargo test --workspace
  ```
  Comprueba `rust-toolchain.toml`.

- **`cargo fmt --all` reformatea todo:** no ejecutarlo sin filtrar; usa
  `cargo fmt` solo sobre archivos tocados.

## 9. Qué hacer cuando tengas `game_scarlett_master.exe` legal

1. Coloca el ejecutable y sus DLLs en una carpeta (fuera del repo).
2. Ejecuta la traducción previa:
   ```bash
   ./target/release/victory translate --cache --validate \
     <dir>/shaders.rpf <dir>/<title>.exe \
     <dir>/XboxBvhBuilder.dll <dir>/amd_fidelityfx_upscaler_xbox.dll
   ```
3. Ejecuta:
   ```bash
   ./target/release/victory run <dir>/<title>.exe --execute
   ```
   Nota: stage 2 seguirá bloqueado hasta portar el loader Linux; la
   traducción y el smoke Vulkan sí funcionan.

Aclaración legal: no distribuyas ni commitees ejecutables, shaders ni
assets del juego. El usuario aporta su copia legal. Ver
`docs/asset-policy.md`.

## Scripts reproducibles

- `scripts/reproduce-all.sh` — ejecuta todos los pasos verificables.
- `scripts/reproduce-troubleshooting.sh` — diagnóstico de fallos.
- `scripts/verify-all.sh` — build + tests + gpu-check + test-vulkan.

```bash
./scripts/reproduce-all.sh
./scripts/reproduce-troubleshooting.sh
```