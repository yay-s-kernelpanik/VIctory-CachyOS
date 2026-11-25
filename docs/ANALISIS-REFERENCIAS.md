# Análisis de repositorios de referencia

**Fecha:** 2026-11-25
**Estado:** análisis basado en la tabla aportada por el usuario.

> **Caveat importante:** los repositorios de referencia **no están presentes
> en este entorno** (no se recibieron rutas ni clones). No se pudo leer su
> código, sus scripts ni sus logs. Por tanto, lo que sigue distingue
> explícitamente entre **"declarado"** (según la tabla del usuario) y
> **"verificable"** (lo que podemos comprobar en nuestro repo). No se asume
> que las afirmaciones de los repos sean ciertas.

---

## 1. `fishingpvalues/gta6-pc-port-hub`

| Aspecto | Declarado | Verificable en nuestro repo |
|---|---|---|
| Rol | Hub de coordinación: inventario de paquetes, censo de shaders, bloqueos, licencias | — |
| Estado | **Nada bootea**: el paquete `1.0.2.1` no contiene `game_scarlett_master.exe` | Nuestro `victory run` tampoco carga título en Linux |
| Técnica clave | Diagnóstico de procedencia: cinco proyectos llegan a la misma conclusión | Podemos replicar el chequeo: buscar el ejecutable en un paquete y reportar ausencia |

**Qué imitar en Linux/CachyOS:**
- Un comando de diagnóstico que, dado un directorio de paquete, compruebe la
  presencia del ejecutable y de los archivos mínimos. En nuestro caso ya
  existe `victory inspect` para un PE concreto; falta un chequeo de paquete.
- Publicar un *hardware report* de CachyOS cuando `gpu-check` pase (hito 5).

**Bloqueo:** sin ejecutable legal, ningún proyecto puede bootear. Es un
problema de procedencia, no de código. No se debe prometer ejecución del
juego.

---

## 2. `lynxnathan/rage-tools-vi`

| Aspecto | Declarado | Verificable en nuestro repo |
|---|---|---|
| Rol | Extracción RPF9, 188,298 entradas, 22 archivos, 161.3 GB | — |
| Resuelto | Cobertura de payload, límites de entradas, metadatos de script, layout de pack | — |
| No resuelto | Nombres originales, bytecode de script | — |
| Técnica clave | Verificación por checksum zlib de cada payload; reconstrucción de nombres por JOAAT (36,952 pares) | Nuestro `victory-shader`/`census` no toca RPF |

**Qué imitar en Linux/CachyOS:**
- Las herramientas son Python puro; corren en CachyOS sin cambios.
- Usar `stream_index.py` / `extract_entry.py` / `script_meta.py` para
  **verificar assets** antes de cargarlos.
- Usar `names.tsv` como diccionario para mapear shaders y assets.

**Bloqueo:** no tenemos el repo ni los assets. Documentado como pendiente de
integración.

---

## 3. `ThatRecompDev/five-star-recomp`

| Aspecto | Declarado | Verificable |
|---|---|---|
| Rol | (Sin README aportado) | — |
| Técnica | Recompilación estática (por el nombre) | No verificable |
| Estado | Pendiente de auditar | — |

**Acción:** clonar y leer README; buscar (a) cómo recompila, (b) a qué
formato traduce shaders, (c) si tiene tests sin datos del juego. Hasta
entonces, no se usa como referencia técnica.

---

## 4. `R0Y00/Xbox2Windows`

| Aspecto | Declarado | Verificable en nuestro repo |
|---|---|---|
| Rol | Recompilación estática x64 + traducción de shaders gfx1013 → DXIL | Nuestro `victory-shader` ya traduce GFX10 → DXIL |
| Cobertura | 26,500/26,503 shaders con ruta DXIL | Nuestro README declara cifras similares para el mundo |
| GPU check | `Run-GpuCheck.ps1`: root signatures, compute, BVH en hardware real, sin datos del juego | Nuestro `gpu-check` cubre device/surface/swapchain/clear/present; **falta compute/BVH** |
| Tests | Diferenciales byte a byte (nativa, mapeada, recompilada) | Nuestros tests son unitarios; no hay diferenciales de shaders en Linux |
| Estado | Startup del EXE original no verificado; intro reproducida con EXE de diagnóstico | Nuestro `run` no ejecuta el PE en Linux |

**Qué imitar en Linux/CachyOS:**
1. **El GPU check sin juego** es la idea más valiosa. Nuestro
   `victory gpu-check` ya es el primer paso; el siguiente es añadir un
   dispatch compute y, si aplica, queries BVH.
2. **La traducción de shaders a un formato de escritorio**: ellos emiten
   DXIL; nosotros necesitamos **SPIR-V** para Vulkan. Ya añadimos
   `dxil-spirv` y `victory_shader::spirv`; falta integrarlo en `translate`.
3. **Tests diferenciales**: comparar la salida del traductor con una
   referencia. Requiere un corpus de shaders; no disponible.

**Bloqueo:** sin su repo no se puede confirmar el 26,500/26,503 ni el
`Run-GpuCheck.ps1`. Se toma como hipótesis de trabajo, no como hecho.

---

## 5. `VIctory-CachyOS` (nuestro repo)

| Aspecto | Estado real | Evidencia |
|---|---|---|
| Compila en Linux | ✅ | `cargo build --release -p victory` |
| Tests | ✅ 118+ | `cargo test --workspace` |
| GPU Vulkan smoke | ✅ | `victory gpu-check` → PASS |
| SPIR-V (DXIL→SPIR-V) | ⚠️ módulo + tests, **no CLI** | `victory_shader::spirv` |
| `translate` Linux | ❌ gated | `main.rs` |
| `run` de título Linux | ❌ solo smoke | `main.rs` |

---

## 6. Tabla resumen: técnica → acción en Linux

| Repo | Técnica clave (declarada) | Acción concreta en VIctory-CachyOS | Estado |
|---|---|---|---|
| port-hub | Diagnóstico de procedencia | chequeo de paquete/ejecutable | pendiente |
| rage-tools-vi | Verificación de assets por checksum + JOAAT | integrar scripts Python | bloqueado (sin repo) |
| five-star-recomp | Recompilación estática | auditar README | pendiente |
| Xbox2Windows | GPU check + shaders a formato PC | `gpu-check` (hecho), SPIR-V (parcial) | parcial |
| VIctory | Host D3D12 | espejo Vulkan | smoke hecho |

---

## 7. Conclusión

La ventaja real de los repos de referencia no es "ejecutar GTA VI": es que
**separan el trabajo que no necesita el juego** (GPU check, extracción y
verificación de assets, traducción de shaders) del que sí lo necesita
(cargar el ejecutable). Ese es el plan que adoptamos: avanzar todo lo
posible sin datos legales y dejar el hito de ejecución bloqueado hasta que
el usuario aporte una copia legal.
