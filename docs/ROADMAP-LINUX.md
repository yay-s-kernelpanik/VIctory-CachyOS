# Roadmap Linux — VIctory-CachyOS

**Actualizado:** 2026-11-25
**Rama:** `feat/vulkan-minimo`

## Estado de hitos

| Hito | Descripción | Estado | Verificación |
|---|---|---|---|
| 0 | Sistema estable (kernel/firmware, sin resets) | ⚠️ No ejecutado | `uname -r`, `dmesg` |
| 1 | Vulkan mínimo: ventana + clear + present | ✅ Hecho | `victory test-vulkan` |
| 2 | GPU check sin datos del juego | ✅ Hecho | `victory gpu-check` |
| 3 | Shaders DXIL → SPIR-V en `translate` | ⚠️ Parcial (módulo + tests; CLI no) | `cargo test -p victory-shader spirv` |
| 4 | Verificar assets (rage-tools-vi) | ⛔ Bloqueado (repo/assets ausentes) | `stream_index.py` |
| 5 | Primer frame del menú (PE de prueba) | ⛔ Bloqueado (loader Linux + Hito 3) | `victory run <pe> --execute` |
| 6 | Juego real con datos legales | ⛔ Bloqueado (sin ejecutable) | `victory run <game> --execute` |

## Lo que funciona hoy en CachyOS

```bash
victory inspect <file>       # análisis PE
victory census <paths>       # agregación de imports
victory shader-info <paths>  # contenedores de shader
victory lib-exports <lib>    # exports de DLL
victory config               # configuración
victory test-vulkan          # ventana + clear + present
victory gpu-check            # informe de GPU + PASS/FAIL
```

## Lo que NO funciona hoy en Linux

```bash
victory translate ...        # gated en Windows (D3D12)
victory run <exe> --execute  # solo smoke Vulkan; no carga el título
```

## Bloqueos

| Bloqueo | Hito afectado | Workaround |
|---|---|---|
| Sin ejecutable legal | 6 | datos del usuario |
| `translate` gated / sin CLI SPIR-V | 3 | des-gatear + integrar `victory_shader::spirv` |
| Loader/binding de PE Windows-only | 5 | portar `victory_loader`/HLE a Linux |
| Sin repos de referencia | 4 | aportar clones/rutas |
| Sin sudo para ajustes de sistema | 0 | cambios manuales del usuario |

## Próximos pasos concretos

1. **Hito 3 (siguiente):** des-gatear `translate` en Linux, convertir el DXIL
   emitido con `victory_shader::spirv::convert_to_bytes` y escribir `.spv` en
   la caché.
2. **Hito 2+:** añadir un dispatch compute a `gpu-check` usando un shader
   SPIR-V embebido.
3. **Hito 4:** integrar `rage-tools-vi` cuando esté disponible.
4. **Hito 5:** portar la carga de PE (`victory_loader`) a Linux.

## Evidencia

- `docs/AUDITORIA-REAL.md` — estado verificado.
- `docs/PLAN-EJECUCION.md` — hitos y verificaciones.
- `docs/host-api-audit.md` — API del host D3D12.
- `docs/ANALISIS-REFERENCIAS.md` — análisis de repos de referencia.
