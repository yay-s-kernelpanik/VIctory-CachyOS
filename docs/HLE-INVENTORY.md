# HLE inventory for Linux porting

Generated from `crates/victory-hle/src/modules/`. Function counts are
`hle_export!` + `extern "system" fn` per module. Tests are `#[test]` in
the module file. All modules are currently Windows-only
(`#[cfg(windows)]`), so **ported to Linux = 0**.

Total functions: **684** across 28 modules. Ported to Linux: 0 (0%).

| Módulo | Funciones | Tests | Portado a Linux | Notas |
|---|---|---|---|---|
| net | 94 | 1 | ❌ | Networking HLE; heavy extern surface. |
| files | 72 | 5 | ❌ | File I/O HLE. |
| gameinput | 52 | 7 | ❌ | GameInput (stubs in part). |
| xasync | 48 | 7 | ❌ | Async operation HLE. |
| kernel | 39 | 3 | ❌ | Kernel (threads, events, memory) — crítico para loader. |
| xstore | 36 | 1 | ❌ | Store commerce HLE. |
| media | 31 | 2 | ❌ | Media playback. |
| xuser | 30 | 2 | ❌ | User/gamertag HLE. |
| xgamesave | 30 | 7 | ❌ | Save containers. |
| audio | 30 | 5 | ❌ | Audio HLE + capture. |
| xmem | 28 | 10 | ❌ | Memory allocation HLE. |
| xpackage | 23 | 1 | ❌ | Package management. |
| dstorage | 23 | 13 | ❌ | DirectStorage (high test coverage). |
| xgameruntime | 20 | 1 | ❌ | Gameruntime. |
| xsystem | 18 | 4 | ❌ | System info (registry). |
| xg | 18 | 4 | ❌ | XG graphics helpers. |
| xnetworking | 15 | 1 | ❌ | Low-level networking. |
| speech | 12 | 1 | ❌ | Accessibility/speech. |
| xgameui | 11 | 1 | ❌ | Game UI overlay. |
| storage | 10 | 1 | ❌ | Persistent local storage (COM object). |
| dsp | 10 | 1 | ❌ | DSP. |
| apu | 10 | 1 | ❌ | APU. |
| xgame | 9 | 2 | ❌ | Title/events. |
| display | 8 | 1 | ❌ | Display/output (swapchain, present) — crítico. |
| fidelityfx | 5 | 3 | ❌ | FidelityFX upscaler hooks. |
| d3d12x | 1 | 1 | ❌ | D3D12X shader cache hook. |
| mod | 1 | 0 | ❌ | Module registration. |
| lifecycle | 0 | 0 | ❌ | Lifecycle (no HLE exports). |

## Prioridad para el loader Linux (stage 2)

1. **kernel** — threads, events, memory, module loading.
2. **display** — swapchain/present (Vulkan bridge).
3. **storage** — file access for saves/config.
4. **xmem** — memory allocation (needed early).
5. **files / xasync** — file I/O used by asset loading.

El resto puede quedar como stubs que retornan `E_NOTIMPL` hasta que
el título los requiera.