# Roadmap Linux — VIctory-CachyOS

**Actualizado:** 2026-11-25
**Rama:** `feat/vulkan-minimo`

## Estado de hitos

| Hito | Descripción | Estado | Verificación |
|---|---|---|---|
| 0 | Sistema estable (kernel/firmware, sin resets) | ⚠️ No ejecutado | `uname -r`, `dmesg` |
| 1 | Vulkan mínimo: ventana + clear + present | ✅ Hecho | `victory test-vulkan` |
| 2 | GPU check sin datos del juego (incluye compute 1x1x1) | ✅ Hecho | `victory gpu-check` |
| 3 | Shaders DXIL → SPIR-V en `translate` | ✅ CLI emite `.spv`; `--pipelines` pendiente | `victory translate <dir> --cache` |
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
victory gpu-check            # GPU + compute dispatch + PASS/FAIL
victory translate <paths>    # DXIL + SPIR-V (Linux)
victory translate <paths> --cache  # escribe .spv en la caché
```

## Lo que NO funciona hoy en Linux

```bash
victory translate ... --pipelines   # VkPipeline pendiente en Linux
victory run <exe> --execute         # prepara el PE, pero no lo ejecuta
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

1. **Hito 3+:** crear `VkPipeline` desde el SPIR-V en `translate --pipelines`
   (compute primero; graphics después).
2. **Hito 4:** integrar `rage-tools-vi` cuando esté disponible.
3. **Hito 5:** portar el mapeo/carga de PE (`victory_loader`) a Linux.

## Evidencia

- `docs/AUDITORIA-REAL.md` — estado verificado.
- `docs/PLAN-EJECUCION.md` — hitos y verificaciones.
- `docs/host-api-audit.md` — API del host D3D12.
- `docs/ANALISIS-REFERENCIAS.md` — análisis de repos de referencia.
