# Estado de Portabilidad Nativa a Linux/CachyOS

## Dependencias de Windows identificadas
| Archivo/Crate | Dependencia | Alternativa en Linux | Esfuerzo |
|---|---|---|---|
| crates/victory-gpu | d3d12 (Direct3D 12) | ash/vulkano (Vulkan) | Alto |
| crates/victory-gpu | dxgi (DXGI swapchain) | vulkan swapchain | Medio |
| crates/victory-gpu/src/resource.rs | view_format (D3D12) | Vulkan format | Bajo |
| crates/victory-shader/src/emit/host.rs | D3D12 device/pipeline | Vulkan pipeline | Alto |
| crates/victory-shader/src/emit/validator.rs | dxil.dll validator | SPIR-V validator | Medio |
| crates/victory-hle/src/modules/xmem.rs | windows_sys memory mgmt | Linux mmap/mprotect | Medio |
| crates/victory-hle/src/modules/lifecycle.rs | windows_sys process mgmt | Linux process APIs | Bajo |
| crates/victory-hle/src/modules/d3d12x.rs | D3D12 device creation | Vulkan device | Alto |
| crates/victory-hle/src/modules/dstorage.rs | DirectStorage | Vulkan staging | Medio |

## Módulos que compilan en CachyOS
- [x] victory-core
- [x] victory-pe
- [x] victory-loader (Linux ELF loading via dlopen)
- [x] victory-shader (core translation, host/validator gated)
- [x] victory-gpu (Linux Vulkan smoke path en `host_vulkan.rs`; backend completo NO IMPLEMENTADO)
- [x] victory-hle (Windows-specific modules gated, Linux stubs provided)
- [x] victory (CLI binary: inspect, census, shader-info, lib-exports, config)

## Módulos que requieren traducción
- [x] GPU: swapchain Vulkan mínima (clear + present) — verificado con `victory test-vulkan`
- [ ] GPU: pipelines → Vulkan
- [ ] Shaders: DXIL → SPIR-V (partially done via victory-shader)
- [ ] host D3D12 pipeline creation → Vulkan pipeline creation
- [ ] DirectStorage → Vulkan staging buffers

## Comandos disponibles en Linux
- `victory inspect <file>` — PE image analysis
- `victory census <paths>` — import aggregation
- `victory shader-info <paths>` — shader container discovery
- `victory lib-exports <lib>` — DLL export listing
- `victory config` — configuration display
- `victory test-vulkan` — abre ventana, clear color y present (Linux)
- `victory run <exe> --execute` — en Linux llega al smoke Vulkan; no carga el título
- `victory translate` — (gated, requires D3D12)

## Próximos pasos
1. Port D3D12 pipeline creation to Vulkan (sobre `host_vulkan.rs`)
2. Complete DXIL → SPIR-V shader translation
3. Implement DirectStorage equivalent with Vulkan staging
4. Replace the clear with the first real draw
5. Enable `victory run` with title loading on Linux
6. Test with real game data (user-provided)
