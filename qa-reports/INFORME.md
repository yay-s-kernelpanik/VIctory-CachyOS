# Informe de QA — VIctory-CachyOS Nativo en CachyOS

**Fecha:** 2026-11-25
**Commit auditado:** 70564ae + rama feat/vulkan-minimo
**Rama:** feat/vulkan-minimo
**Auditor:** IA (freebuff/Beam 1.0)

---

## Tabla de Resultados por Fase

| Fase | Descripción | Estado | Detalle |
|---|---|---|---|
| 1 | Preparar entorno Rust | ✅ Éxito | rustc 1.98.1, cargo 1.98.1, LLVM 23.1.5 |
| 2 | Compilar proyecto | ✅ Éxito | victory binary compilado en release |
| 3 | Identificar D3D12 | ✅ Documentado | Ver docs/NATIVE-LINUX.md |
| 4 | Verificador de shaders | ⚠️ Parcial | Comando existe pero gated en Linux |
| 5 | Configurar datos de prueba | ⚠️ Parcial | No hay datos del juego; fixtures disponibles |
| 6 | Ejecutar loader con fixtures | ✅ Éxito | census e inspect funcionan |
| 7 | Documentar bloqueos D3D12→Vulkan | ✅ Completo | docs/NATIVE-LINUX.md |
| 8 | Registrar avances en commits | ✅ Completo | Múltiples commits |
| 9 | Script reproducible | ✅ Completo | run-native-tests.sh |

## Módulos que compilan en CachyOS
- victory-core ✅
- victory-pe ✅
- victory-loader ✅ (Linux ELF loading via dlopen)
- victory-shader ✅ (core translation; host/validator gated)
- victory-gpu ✅ (Linux Vulkan smoke path en `host_vulkan.rs`: instance/device/swapchain/clear/present; backend completo **NO IMPLEMENTADO**)
- victory-hle ✅ (Windows-specific modules gated, Linux stubs)
- victory (CLI) ✅ (inspect, census, shader-info, lib-exports, config)

## Tests
- 118+ tests pasados
- 4 tests ignorados (Windows-specific, gated behind cfg(windows))
- 0 fallos

## Comandos disponibles en Linux
- `victory inspect <file>` — análisis de imagen PE
- `victory census <paths>` — agregación de imports
- `victory shader-info <paths>` — descubrimiento de contenedores de shader
- `victory lib-exports <lib>` — listado de exports de DLL
- `victory config` — visualización de configuración

## Comando nuevo: smoke Vulkan
- `victory test-vulkan` — abre ventana, pinta un clear color y presenta un frame; sale sin crash.
- `victory run <exe> --execute` — en Linux llega hasta el mismo smoke Vulkan; **no carga el título** (eso sigue NO FUNCIONAL).

## Comandos gated (requieren D3D12)
- `victory translate` — traducción de shaders (requiere D3D12 host)

## Próximos pasos
1. Portar creación de pipelines D3D12 → Vulkan (sobre `host_vulkan.rs`)
2. Completar traducción DXIL → SPIR-V
3. Implementar DirectStorage equivalente con Vulkan staging
4. Sustituir el clear por el primer draw real del menú
5. Habilitar `victory run` con carga de título en Linux
6. Probar con datos reales del juego (aportados por el usuario)
