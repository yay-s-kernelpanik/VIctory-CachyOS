# Plan de ejecución — VIctory-CachyOS

**Fecha:** 2026-11-25
**Objetivo:** llevar VIctory a ejecutarse de forma nativa en CachyOS Linux
(sin Wine/Proton), avanzando todo lo posible **sin** el ejecutable legal del
juego.

> Regla: cada hito tiene un comando de verificación y su salida. Sin
> evidencia, el hito no cuenta.

---

## Hito 0 — Sistema estable

**Objetivo:** CachyOS sin resets de GPU ni OOM al probar Vulkan.

**Acciones:**
- Kernel LTS/Zen estable (actual: `7.3.4-1-cachyos`).
- `linux-firmware-amdgpu` actualizado.
- Si hay resets: `amdgpu.dcdebugmask=0x10`.

**Verificación:**
```bash
uname -r
dmesg -T | grep -iE 'amdgpu.*(reset|error)|oom'
```
**Estado:** ⚠️ No ejecutado. Requiere cambios de sistema (sudo no
disponible). `dmesg` no mostró errores en esta sesión.

---

## Hito 1 — Backend Vulkan mínimo (clear color)

**Objetivo:** `victory test-vulkan` abre ventana, pinta color y cierra sin
crash.

**Acciones:** hechas.
- `ash`/`ash-window`/`winit` en `victory-gpu` (Linux).
- `crates/victory-gpu/src/host_vulkan.rs`: instancia, device, surface,
  swapchain, command buffers, `vkCmdClearColorImage`, `vkQueuePresentKHR`.

**Verificación:**
```bash
$ ./target/debug/victory test-vulkan
Vulkan ready: 800x600, format B8G8R8A8_SRGB
presented one clear-colour frame
```
**Estado:** ✅ Completado (commit `3a95755` y anteriores).

---

## Hito 2 — GPU check sin datos del juego

**Objetivo:** suite de comprobación de GPU inspirada en `Run-GpuCheck.ps1`,
sin necesitar el juego.

**Acciones:** hechas.
- Comando `victory gpu-check`.
- Reporta device, tipo, versión Vulkan, driver, familia de cola, swapchain.
- Crea un `VkComputePipeline` desde SPIR-V embebido y hace un dispatch 1x1x1.
- Ejecuta clear + present y reporta `PASS`/`FAIL`.

**Verificación:**
```bash
$ ./target/debug/victory gpu-check
GPU check
  device:    AMD Radeon 680M (RADV REMBRANDT) (INTEGRATED_GPU)
  vulkan:    1.4.354
  driver:    0x6802006
  queue:     family 0 (graphics)
  swapchain: 3 images, 800x600, B8G8R8A8_SRGB
GPU check: PASS
```
**Estado:** ✅ Completado (incluye compute dispatch). **Pendiente:** queries BVH.

---

## Hito 3 — Shaders a SPIR-V

**Objetivo:** `victory translate --cache` emite SPIR-V en Linux.

**Acciones:**
- ✅ Añadido `dxil-spirv` (Linux) a `victory-shader`.
- ✅ Añadido `victory_shader::spirv` (`convert`, `convert_to_bytes`,
  `words_to_bytes`) con 3 tests.
- ✅ Integrado en `translate` en Linux; `--cache` escribe los `.spv`.
- ✅ `gpu-check` crea un `VkComputePipeline` desde SPIR-V y hace dispatch.
- ❌ `--pipelines` para todos los shaders (VkPipeline por shader) aún no.

**Verificación:**
```bash
$ cargo test -p victory-shader spirv
test result: ok. 3 passed; 0 failed

$ ./target/debug/victory translate tests/ --cache
SPIR-V       0 shaders converted (Vulkan)
```
**Estado:** ✅ CLI funcionando; `--pipelines` pendiente.

---

## Hito 4 — Verificar assets con rage-tools-vi

**Objetivo:** comprobar checksums y nombres antes de cargar assets.

**Acciones:**
- ❌ Integrar `rage-tools-vi` (no está en el entorno).
- ❌ Ejecutar `stream_index.py` / `extract_entry.py` sobre assets legales.

**Verificación (prevista):**
```bash
python3 stream_index.py --root <rpf-dir>
python3 extract_entry.py <entry>
```
**Estado:** ⛔ Bloqueado: repositorio y assets no disponibles.

---

## Hito 5 — Primer frame del menú

**Objetivo:** `victory run <exe_de_prueba> --execute` llega al menú del PE
de prueba.

**Acciones:**
- ✅ Leer y preparar el PE en Linux (`victory_loader::prepare`).
- ❌ Mapear/ejecutar el PE (loader/binding HLE).
- ❌ Integrar Hitos 1–4.
- ❌ Sustituir el clear por el primer draw real.

**Verificación (prevista):**
```bash
./target/debug/victory run <exe-de-prueba> --execute
```
**Estado:** ⛔ Bloqueado por Hito 3/4 y por la carga de PE en Linux.

---

## Hito 6 — Ejecución con datos reales

**Objetivo:** `victory run <game_exe> --execute` carga el juego real.

**Bloqueo:** **no hay ejecutable legal disponible**. No se puede completar
hasta que el usuario aporte una copia legal.

**Verificación (prevista):**
```bash
./target/debug/victory run <ruta-legal>/game_scarlett_master.exe --execute
```
**Estado:** ⛔ Bloqueado por procedencia.

---

## Orden de dependencias

```
Hito 0 (sistema)
  └─ Hito 1 (Vulkan clear)  ✅
       └─ Hito 2 (gpu-check) ✅
            └─ Hito 3 (SPIR-V) ⚠️ parcial
                 └─ Hito 4 (assets) ⛔
                      └─ Hito 5 (menú) ⛔
                           └─ Hito 6 (juego real) ⛔
```

## Lo que se puede hacer ya (sin ejecutable)

1. Completar Hito 3: des-gatear `translate` y escribir `.spv`.
2. Ampliar `gpu-check` con un dispatch compute.
3. Integrar `rage-tools-vi` si el usuario aporta el repo.
4. Auditar `five-star-recomp` si el usuario aporta el repo.

## Lo que NO se puede hacer todavía

- Ejecutar el juego real (Hito 6).
- Cargar el menú de un PE real (Hito 5) sin completar el loader Linux.
- Verificar assets (Hito 4) sin repositorio/herramientas.
