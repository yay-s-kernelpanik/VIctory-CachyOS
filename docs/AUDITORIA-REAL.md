# Auditoría real — VIctory-CachyOS (Linux/CachyOS)

**Fecha:** 2026-11-25
**Repositorio git real:** `/home/joaac000/VIctory-CachyOS`
**Rama:** `feat/vulkan-minimo`
**Base:** `70564ae` + commits del port nativo
**Entorno:** CachyOS, kernel `7.3.4-1-cachyos`, rustc/cargo 1.98.1, clang/LLVM 23.1.5
**GPU verificada:** AMD Radeon 680M (RADV REMBRANDT), Vulkan 1.4.354

> Este documento reemplaza las afirmaciones optimistas de informes
> anteriores. Todo lo que sigue se verificó contra el código y se ejecutó en
> esta máquina; lo que no se pudo verificar se marca como tal.

---

## 1. Qué existe y qué no

| Elemento | Estado real | Evidencia |
|---|---|---|
| `crates/victory-gpu/src/host_linux.rs` | **NO EXISTE** | `ls` → "No such file or directory" |
| `crates/victory-gpu/src/host_vulkan.rs` | **EXISTE** (smoke path) | instance/device/swapchain/command/clear/present |
| Dependencias Vulkan | **SÍ** (`ash` 0.38, `ash-window` 0.13, `winit`, `raw-window-handle`) solo Linux | `crates/victory-gpu/Cargo.toml` |
| Backend Vulkan completo (pipelines, shaders, descriptores) | **NO IMPLEMENTADO** | solo clear color |
| `victory test-vulkan` | **FUNCIONAL** | ver §3 |
| `victory gpu-check` | **FUNCIONAL** | ver §3 |
| `victory translate` en Linux | **FUNCIONAL** para DXIL + conversión a SPIR-V; `--pipelines` aún no | `main.rs` → `translate::run_linux` |
| `victory run <exe> --execute` en Linux | Lee y **prepara** el PE, luego smoke Vulkan; **no mapea ni ejecuta** el título | `run_linux` |
| `victory inspect/census/shader-info/lib-exports/config` | **FUNCIONAL** | `victory --help` |
| Datos legales del juego | **NO DISPONIBLES en el entorno** | no hay ejecutable del título |

---

## 2. Compilación y tests

```bash
$ cargo build --release -p victory
Finished `release` profile ... in 29.77s

$ cargo test --workspace
test result: ok. 118 passed; 0 failed; 1 ignored
test result: ok. 23 passed; 0 failed; 4 ignored
...
0 failed

$ cargo test -p xtask policy
test result: ok. 4 passed; 0 failed
```

- Los 4 tests ignorados son Windows-specific (PE parsing, kernel module
  handle, prepare module, storage `GetFileAttributesExW`).
- Los logs de QA se movieron a `~/logs-victory/` para cumplir la política de
  assets; `.log` sigue en la allowlist.

---

## 3. Verificación del host Vulkan

```bash
$ ./target/debug/victory gpu-check
GPU check
  device:    AMD Radeon 680M (RADV REMBRANDT) (INTEGRATED_GPU)
  vulkan:    1.4.354
  driver:    0x6802006
  queue:     family 0 (graphics)
  swapchain: 3 images, 800x600, B8G8R8A8_SRGB
Vulkan ready: 800x600, format B8G8R8A8_SRGB
presented one clear-colour frame
GPU check: PASS
```

```bash
$ ./target/debug/victory test-vulkan
Vulkan ready: 800x600, format B8G8R8A8_SRGB
presented one clear-colour frame
```

```bash
$ ./target/debug/victory run /bin/true --execute
title          /bin/true
Vulkan ready: 800x600, format B8G8R8A8_SRGB
presented one clear-colour frame
```

Salida sin crash. El warning `xdg_toplevel_icon_manager_v1 is not supported`
es de winit/Wayland y no afecta al present.

---

## 4. Qué hace realmente el "run" de Linux

`victory run <exe> --execute` en Linux **lee y prepara** el PE con
`victory_loader::prepare` (manipulación de bytes portable: marca DLL, quita
entry point e imports, limpia TLS callbacks) y reporta el build. Después llama
al smoke Vulkan (`victory_gpu::host_vulkan::run()`). **No mapea ni ejecuta** la
imagen: el flujo de carga (`load`, binding de imports, `spawn_main`) sigue
siendo Windows-only. Por tanto:

- **NO** se puede afirmar que VIctory ejecute GTA VI en Linux.
- **SÍ** se puede afirmar que el host Vulkan mínimo (instancia, device,
  swapchain, clear, present) funciona en CachyOS.

---

## 5. Shaders

- `victory-shader` compila en Linux y traduce shaders de consola a DXIL.
- Se añadió `dxil-spirv` (Linux) y `victory_shader::spirv` para convertir
  DXIL → SPIR-V. Tests: 3 passed (`empty`, `garbage`, `round-trip`).
- **Integrado** en `victory translate` en Linux: cada shader emitido se
  convierte y se cuenta; con `--cache` se escriben los `.spv` en
  `<cache>/shaders`.
- `--pipelines` en Linux aún no crea `VkPipeline` (pendiente).

---

## 6. Repositorios de referencia

No están presentes en el entorno (no se adjuntaron rutas). El análisis de
`docs/ANALISIS-REFERENCIAS.md` se basa en la tabla aportada por el usuario y
**no se pudo verificar contra su código**. Se marca cada afirmación como
"declarado" cuando no hay evidencia local.

---

## 7. Bloqueos reales

| Bloqueo | Impacto | Workaround |
|---|---|---|
| Sin ejecutable legal del juego | No se puede probar carga real | fixtures + `gpu-check` + datos del usuario |
| `translate` y `run` de título gated | No hay pipeline de shaders/juego en Linux | des-gatear + integrar SPIR-V + cargar PE |
| Backend Vulkan solo clear | No dibuja geometría | pipelines, shaders, descriptores |
| Repos de referencia no adjuntos | No se puede auditar su código | aportar rutas/clones |

---

## 8. Comandos de verificación

```bash
cd ~/VIctory-CachyOS
cargo test --workspace
cargo test -p xtask policy
cargo run -p victory -- gpu-check
cargo run -p victory -- test-vulkan
cargo run -p victory -- run /bin/true --execute
```
