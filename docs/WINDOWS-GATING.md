# Windows gating inventory (`cfg(windows)`)

Generated from:

```sh
grep -rn "cfg(windows)\|cfg!(windows)" --include="*.rs" crates/ apps/ > /tmp/gating.txt
```

Total gate sites: 238 across 32 files.

## `crates/victory-gpu` (D3D12.X host — not ported to Vulkan)

| Archivo | Líneas cfg(windows) | Qué contiene (1 frase) | Prioridad |
|---|---|---|---|
| `src/list.rs` | 34 | Command list recording and D3D12 command list wrappers. | Alta |
| `src/pipeline.rs` | 33 | Pipeline state objects and pipeline library management. | Alta |
| `src/resource.rs` | 32 | GPU resource (buffer/texture) creation and views. | Alta |
| `src/descriptor.rs` | 23 | Descriptor heap and descriptor table management. | Alta |
| `src/lib.rs` | 15 | Crate root with host module wiring. | Media |
| `src/queue.rs` | 5 | Command queue submission and fence signaling. | Alta |
| `src/root.rs` | 4 | Root signature layout. | Media |
| `src/fence.rs` | 2 | Fence synchronization primitives. | Media |
| `src/device.rs` | 2 | Device creation and capability queries. | Alta |
| `src/object.rs` | 1 | Generic GPU object handle. | Baja |

## `crates/victory-hle` (Xbox HLE — 100% Windows)

| Archivo | Líneas cfg(windows) | Qué contiene (1 frase) | Prioridad |
|---|---|---|---|
| `src/modules/mod.rs` | 18 | Module registration and dispatch table. | Alta |
| `src/modules/display.rs` | 4 | Display/output HLE (swapchain, present). | Alta |
| `src/modules/d3d12x.rs` | 3 | D3D12X-specific HLE thunks. | Media |
| `src/modules/kernel.rs` | 2 | Kernel HLE (threads, events, memory). | Alta |
| `src/modules/storage.rs` | 2 | Storage/file HLE. | Alta |
| `src/modules/gameinput.rs` | 2 | GameInput HLE. | Media |
| `src/modules/xgameui.rs` | 2 | Game UI overlay HLE. | Baja |
| `src/modules/xsystem.rs` | 2 | System info HLE. | Media |
| `src/modules/xgameruntime.rs` | 1 | Gameruntime HLE. | Media |
| `src/lib.rs` | 2 | Crate root. | Media |
| `src/host.rs` | 1 | Host trait definition. | Media |
| `src/runtime.rs` | 1 | Runtime state container. | Media |
| `tests/fidelityfx.rs` | 1 | FidelityFX test harness. | Baja |
| `tests/no_dll_planting.rs` | 1 | DLL planting guard test. | Baja |

## `crates/victory-loader` (PE loader — execution blocked on Linux)

| Archivo | Líneas cfg(windows) | Qué contiene (1 frase) | Prioridad |
|---|---|---|---|
| `src/lib.rs` | 9 | `load`/`load_title_dll(s)` entry points (Windows-only). | Alta |
| `src/prepare.rs` | 1 | Import analysis helpers (partially portable). | Media |
| `tests/title_dlls.rs` | 1 | Title DLL loading test. | Baja |
| `tests/load_probe.rs` | 1 | Load probe test. | Baja |

## `crates/victory-shader` (shader emitter)

| Archivo | Líneas cfg(windows) | Qué contiene (1 frase) | Prioridad |
|---|---|---|---|
| `src/emit/mod.rs` | 8 | Host-side D3D12 emission helpers. | Media |
| `src/native.rs` | 1 | Native host detection. | Media |

## `apps/victory` (CLI)

| Archivo | Líneas cfg(windows) | Qué contiene (1 frase) | Prioridad |
|---|---|---|---|
| `src/translate.rs` | 13 | Platform dispatch for translate (DXIL vs SPIR-V). | Media |
| `src/run.rs` | 6 | Platform dispatch for run (loader vs Vulkan smoke). | Media |
| `src/main.rs` | 6 | Platform-gated module wiring. | Media |

## Resumen

- Total archivos gateados: 32.
- Crates más afectados: `victory-gpu` (10 archivos, 151 sites), `victory-hle` (14 archivos, 40 sites), `victory-loader` (4 archivos, 12 sites).
- El loader Linux no puede progresar hasta portar `victory-loader/src/lib.rs` y los módulos `kernel`/`display`/`storage` de `victory-hle`.