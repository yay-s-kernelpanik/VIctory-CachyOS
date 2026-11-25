# Auditoría general archivo por archivo — VIctory (Linux port)

**Rama:** `feat/vulkan-minimo` · **HEAD:** `eba42af` · **Tests:** 260 passed / 0 failed / 5 ignored · **Build:** OK

**Leyenda:** ✅ funciona en Linux · 🟡 parcial · 🔴 Windows-only/bloqueado · ⚪ stub/no aplica · 🧪 tests · ⚠️ error/limitación

---

## 1. `apps/victory` (CLI)

| Archivo | Estado | Notas |
|---|---|---|
| `Cargo.toml` | ✅ | Deps + `tempfile` añadido. |
| `main.rs` | 🟡 | Dispatch por plataforma; `mod rpf9` gateado Linux. Help `--validate` multiplataforma. |
| `translate.rs` | 🟡 | DXIL→SPIR-V, `--cache`, `--validate` (spirv-val), contadores por etapa, notas RPF/>2GiB, preprocess RPF9. **Falta:** `--pipelines` (VkPipeline). |
| `rpf9.rs` | ✅ | Nuevo. Detección magic + extractor externo + stub. 3 tests. |
| `run.rs` | 🟡 | 3 stages; stage 2 BLOCKED (loader Windows). Stage 3 smoke window OK. |
| `shader.rs` | ✅ | `MAX_FILE = 2 GiB`, fuentes. |
| `inspect.rs` | ✅ | Análisis PE. |
| `census.rs` | ✅ | Conteo de shaders. |
| `prebuild.rs` | ⚪ | Windows pipeline prebuild; Linux no. |
| `gpu_check.rs` | ✅ | Device/swapchain/compute dispatch, PASS. |

## 2. `crates/victory-core`

| Archivo | Estado | Notas |
|---|---|---|
| `Cargo.toml` | ✅ | |
| `lib.rs` | ✅ | |
| `config.rs` | ✅ | Rutas, cache. |
| `policy.rs` | ✅ | 4 tests. |
| `logging.rs` | ✅ | |

## 3. `crates/victory-pe`

| Archivo | Estado | Notas |
|---|---|---|
| `lib.rs` | ✅ | |
| `image.rs` | ✅ | PE32+ parse, imports, delay-imports, secciones. Verificado con PE real. |
| `implib.rs` | ✅ | |
| `layout.rs` | ✅ | |

## 4. `crates/victory-game`

| Archivo | Estado | Notas |
|---|---|---|
| `lib.rs` | ✅ | |
| `cityhash.rs` | ✅ | |
| `sgd5.rs` | ✅ | |
| `shaderpack.rs` | ✅ | |
| `pipelines.rs` | ✅ | |

## 5. `crates/victory-formats`

| Archivo | Estado | Notas |
|---|---|---|
| `lib.rs` | ✅ | |
| `bcpack.rs` | ✅ | |
| `texture.rs` | ✅ | |
| `xma.rs` / `xma/tables.rs` | ✅ | |

## 6. `crates/victory-pipeline`

| Archivo | Estado | Notas |
|---|---|---|
| `lib.rs` | ✅ | |
| `cache.rs` | ✅ | Content-addressed cache. |

## 7. `crates/victory-shader` (34 archivos)

| Archivo | Estado | Notas |
|---|---|---|
| `lib.rs` | ✅ | |
| `native.rs` | 🟡 | 1 `cfg(windows)` (host-specific). |
| `spirv.rs` | ✅ | `convert`, `convert_to_bytes`, `words_to_bytes`; 3 tests + 2 nuevos en translate. |
| `dxbc.rs` | ✅ | |
| `dxil.rs` | ✅ | |
| `io.rs`, `ir.rs`, `range.rs`, `resource.rs`, `structure.rs` | ✅ | |
| `gfx10/*` (4) | ✅ | |
| `emit/mod.rs` | 🟡 | 9 `cfg(windows)` (host D3D12). Emite VS/PS/CS/MS. **No emite Gs/Ls/Hs/Es** (195 programas). |
| `emit/lower.rs` (+ `lower/bvh.rs`) | 🟡 | Solo `Vs/Ps/Cs/Ms`. |
| `emit/host.rs` | 🔴 | Host D3D12. |
| `emit/container.rs`, `rect.rs`, `sampler.rs`, `shader.rs`, `bitcode.rs`, `validator.rs`, `wave.rs` | ✅ | |
| `emit/*_tests.rs` (9) | 🧪 | Tests del emisor. |
| `build.rs` | ✅ | |

## 8. `crates/victory-gpu` (35 archivos) — D3D12.X host

| Archivo | Estado | Notas |
|---|---|---|
| `lib.rs` | 🟡 | 1 `cfg(windows)`. |
| `host_vulkan.rs` | 🟡 | Vulkan host mínimo: device, swapchain, compute dispatch, present. **Sin render pass real, sin pipelines gráficos, sin gestión completa de memoria.** |
| `host.rs` | 🔴 | Host D3D12 (Windows). |
| `device.rs`, `queue.rs`, `fence.rs`, `list.rs`, `object.rs`, `resource.rs`, `root.rs`, `descriptor.rs`, `descriptor/host.rs` | 🔴 | `cfg(windows)`; D3D12. **No portados a Vulkan.** |
| `pipeline.rs`, `pipeline/host.rs`, `pipeline/library.rs`, `pipeline/make.rs` | 🔴 | D3D12 pipeline library. **Sin VkPipeline.** |
| `bindless.rs`, `capture.rs`, `console.rs`, `copystat.rs`, `draw.rs`, `dxgi.rs`, `external.rs`, `frame.rs`, `indirect.rs`, `memory.rs`, `primitive.rs`, `query.rs`, `sampler.rs`, `shader_cache.rs`, `shadow.rs`, `signature.rs`, `texture.rs`, `twin.rs`, `watch.rs` | 🔴 | D3D12 host. **Bloqueados.** |

**Resumen gpu:** solo `host_vulkan.rs` aporta Vulkan; el resto del crate es D3D12.X y no está portado.

## 9. `crates/victory-hle` (33 + 29 módulos)

| Archivo | Estado | Notas |
|---|---|---|
| `lib.rs`, `host.rs`, `runtime.rs`, `registry.rs` | 🔴 | `cfg(windows)`. |
| `modules/mod.rs` | 🔴 | Gateado. |
| `modules/kernel.rs`, `display.rs`, `d3d12x.rs`, `storage.rs`, `xgameruntime.rs`, `xsystem.rs`, `gameinput.rs`, `xgameui.rs` | 🔴 | `cfg(windows)`. |
| Resto de módulos (apu, audio, dsp, dstorage, fidelityfx, files, lifecycle, media, net, speech, xasync, xgame, xgamesave, xg, xmem, xnetworking, xpackage, xstore, xuser, xgame*) | 🔴 | Implementan API Xbox; dependen de Windows. **No portados.** |
| `tests/*` (2) | 🧪 | `fidelityfx.rs`, `no_dll_planting.rs`. |

**Resumen hle:** 100% Windows. Bloquea el stage 2 (imports del título).

## 10. `crates/victory-loader` (9 archivos)

| Archivo | Estado | Notas |
|---|---|---|
| `lib.rs` | 🔴 | `load`, `load_title_dll(s)` gateados Windows. |
| `prepare.rs` | 🟡 | `cfg(windows)` en línea 244; análisis de imports multiplataforma parcial. |
| `resolve.rs` | ✅ | Resolución de imports. |
| `crash.rs`, `stacks.rs`, `threads.rs`, `thunks.rs`, `tls.rs`, `windows.rs` | 🔴 | Windows-only. |
| `tests/*` (2) | 🧪 | `load_probe.rs`, `title_dlls.rs`. |

**Bloqueo crítico del proyecto.**

## 11. `xtask`

| Archivo | Estado | Notas |
|---|---|---|
| `Cargo.toml`, `main.rs`, `policy.rs` | ✅ | 4 tests policy. |

## 12. `docs` / `README`

| Archivo | Estado | Notas |
|---|---|---|
| `AUDITORIA-REAL.md`, `PLAN-EJECUCION.md`, `ROADMAP-LINUX.md` | ✅ | Actualizados. |
| `asset-policy.md`, `architecture.md` | ✅ | |
| `README.md` | ✅ | Estado Linux + RPF9. |

---

## Errores / limitaciones detectadas

1. **`victory-gpu` (35 archivos) es en su mayoría D3D12.X no portado.** Solo `host_vulkan.rs` aporta Vulkan; el resto (`device`, `queue`, `pipeline`, `memory`, `texture`, `draw`, etc.) siguen con `cfg(windows)`. **Sin render pass real ni pipelines gráficos.**
2. **`victory-loader::load` Windows-only** → stage 2 BLOCKED. No se mapea/ejecuta el PE.
3. **`victory-hle` 100% Windows** → imports del título no se resuelven en Linux.
4. **Shaders de geometría (`Gs`/`Ls`/`Hs`/`Es`)** no se emiten (195 programas). Emisor solo VS/PS/CS/MS.
5. **`--pipelines` en Linux** pendiente (VkPipeline).
6. **RPF9**: solo stub (extractor externo ausente en el entorno). Sin datos reales no se valida la conversión masiva.
7. **Sin ejecutable legal del juego** → no se puede verificar stage 1 con imports reales ni stage 3 con frames reales.
8. **`cargo fmt --all`** reformatea ~50 archivos no relacionados; se revirtió. No ejecutar sin filtrar.
9. **`write_file` deja línea `EOF`** ocasionalmente; limpiar con `sed`.
10. **`winit` warning** `xdg_toplevel_icon_manager_v1` inofensivo.
11. **Descargas copy** sin `.git`; sincronizada manualmente.

## Porcentaje por subsistema

| Subsistema | Archivos | Portado Linux |
|---|---|---|
| CLI (`apps/victory`) | 9 | ~85% |
| `victory-core` | 4 | 100% |
| `victory-pe` | 4 | 100% |
| `victory-game` | 5 | 100% |
| `victory-formats` | 5 | 100% |
| `victory-pipeline` | 2 | 100% |
| `victory-shader` | 34 | ~70% (VS/PS/CS/MS; sin geometría) |
| `victory-gpu` | 35 | ~5% (solo smoke Vulkan) |
| `victory-hle` | 33+29 mods | 0% |
| `victory-loader` | 9 | ~20% (análisis; sin ejecución) |
| `xtask` | 3 | 100% |
| `docs` | 4+ | 100% |

**Global: ~8/100 puntos del roadmap** (preparación ~80% completa; ejecución del juego bloqueada en stage 2).

## Próximos pasos recomendados

1. Portar `victory_loader::load` a Linux (mapeo + relocations + IAT) → desbloquea stage 2.
2. Portar `victory-gpu` D3D12→Vulkan (pipelines, memoria, render pass) → primer frame.
3. Portar `victory-hle` módulos críticos (kernel, display, storage) → imports resueltos.
4. Implementar `--pipelines` con VkPipeline.
5. Soporte de shaders de geometría en el emisor (si el corpus lo requiere).
6. Validar con `game_scarlett_master.exe` legal cuando esté disponible.