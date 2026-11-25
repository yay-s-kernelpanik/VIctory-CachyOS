# Auditoría de la API de `crates/victory-gpu/src/host.rs` (host D3D12)

Fecha: 2026-11-25
Alcance: `crates/victory-gpu/src/host.rs` (2586 líneas), el único host GPU existente.
Objetivo: documentar la superficie exacta que un espejo Vulkan (`host_vulkan.rs`) debe replicar.

> Nota: el crate no expone nada de esto fuera de `victory-gpu`: todo es
> `pub(crate)` o privado. `lib.rs` reexporta solo `on_failure` y los símbolos
> de `watch` bajo `#[cfg(windows)]`.

---

## 1. Tipos principales

### `pub(crate) struct Host` (línea 38)

```rust
pub(crate) struct Host {
    pub(crate) device: ID3D12Device,
    queue: ID3D12CommandQueue,
    fence: ID3D12Fence,
    submitted: Mutex<u64>,      // último valor señalado
    event: HANDLE,
    rtvs: ID3D12DescriptorHeap,
    rtv_size: u32,
    dsvs: ID3D12DescriptorHeap,
    dsv_size: u32,
    next_dsv: AtomicU32,
    cpu_views: ID3D12DescriptorHeap,
    cpu_view_size: u32,
    next_cpu_view: AtomicU32,
    messages: Option<ID3D12InfoQueue>,
    next_rtv: AtomicU32,
    presenter: Mutex<Option<Presenter>>,
}
```

`unsafe impl Send for Host`, `unsafe impl Sync for Host`.

Equivalente Vulkan mínimo: `VkInstance`, `VkPhysicalDevice`, `VkDevice`,
`VkQueue` gráfica, `VkSurfaceKHR`, swapchain, render pass, command pool,
command buffers, semáforo/fence. `Host.device` es el único campo `pub(crate)`;
el resto es encapsulado.

### `pub(crate) struct HostList` (línea 312)

```rust
pub(crate) struct HostList {
    allocator: ID3D12CommandAllocator,
    pub(crate) list: ID3D12GraphicsCommandList,
    deferred: Mutex<Deferred>,
    scratch: Mutex<Scratch>,
    states: Mutex<States>,
    twins: Mutex<HashMap<usize, crate::twin::Use>>,
    rebind: AtomicBool,
    prelude: (ID3D12CommandAllocator, ID3D12GraphicsCommandList),
    pipelines: Mutex<BTreeSet<String>>,
    budget: OnceLock<Option<(ID3D12Resource, usize)>>,
    // diagnostics:
    notes: Mutex<Vec<Note>>,
    targets: Mutex<Vec<(usize, String)>>,
    stamps: OnceLock<Option<(ID3D12QueryHeap, ID3D12Resource)>>,
}
```

Equivalente Vulkan: `VkCommandPool` + `VkCommandBuffer` (y un fence por
submission si se quiere esperar).

### `struct Presenter` (línea 1884, privado)

```rust
struct Presenter {
    swap_chain: IDXGISwapChain3,
    width: u32,
    height: u32,
    format: DXGI_FORMAT,
    copy: HostList,
    hwnd: HWND,
    meter: Meter,
}
```

`struct Meter` (privado) mide FPS y escribe el título de la ventana. No es
necesario para el mínimo Vulkan.

---

## 2. Funciones a nivel de módulo

| Firma exacta | Qué hace |
|---|---|
| `pub(crate) fn pc_state(state: u32) -> u32` | Enmascara bits de compresión de la consola (`0x01ff_ffff`). |
| `pub(crate) fn texture_state(state: u32) -> u32` | `pc_state` sin estados solo-de-buffer. |
| `pub fn on_failure(report: fn(&str) -> !)` | Registra el manejador de fallo del host (crash report). |
| `pub(crate) fn fail(what: std::fmt::Arguments) -> !` | Reporta el fallo y aborta (o llama `on_failure`). |
| `pub(crate) fn host() -> Option<&'static Host>` | **Punto de entrada**: crea el `Host` la primera vez. `None` sin GPU. |
| `pub(crate) fn dred_on() -> bool` | `VICTORY_D3D12_DRED` en builds diagnostics. |
| `pub(crate) fn name(object: &impl Interface, text: impl FnOnce() -> String)` | Pone nombre a un objeto D3D12 (diagnostics). |
| `pub(crate) fn copy_mapped(dst: *mut u8, src: *const u8, len: usize, title_dst: bool)` | Copia entre memoria mapeada; `title_dst` evita lecturas de título. |
| `pub(crate) fn read_into(src: *const u8, dst: &mut [u8]) -> bool` | Lee memoria de título con tolerancia a fallos de página. |
| `pub(crate) fn write_mapped(dst: *mut u8, src: *const u8, len: usize)` | Escribe memoria mapeada. |
| `pub(crate) fn legal_state(texture: &ID3D12Resource, state: u32) -> u32` | Ajusta un estado a los bits válidos del host. |
| `pub(crate) fn legal_for(flags: u32, state: u32) -> u32` | Estado legal según flags del recurso. |
| `pub(crate) fn retire(resource: ID3D12Resource)` | Encola un recurso para liberarlo tras la submission. |
| `pub(crate) fn release_retired()` | Libera los recursos retirados. |
| `pub(crate) fn transition(resource: &ID3D12Resource, before: D3D12_RESOURCE_STATES, after: D3D12_RESOURCE_STATES) -> D3D12_RESOURCE_BARRIER` | Construye un barrier de transición. |

Privadas relevantes: `create() -> windows::core::Result<Host>` (línea 139),
`warp_adapter()`, `make_window(width, height) -> windows::core::Result<HWND>`
(línea 2428), `window(width, height) -> windows::core::Result<HWND>` (hilo
propio), `pump()`, `title_window() -> Option<HWND>`.

---

## 3. `impl Host` (líneas 471, 1791, 2031, 2209)

| Firma exacta | Qué hace |
|---|---|
| `pub(crate) fn mesh_shaders(&self) -> bool` | Soporte de mesh shaders SM 6.6. |
| `pub(crate) fn heap_atomics64(&self) -> bool` | Atómicos 64-bit en heap de descriptores. |
| `pub(crate) fn new_list(&self) -> windows::core::Result<HostList>` | Crea allocator + command list (+ prelude). |
| `pub(crate) fn execute(&self, runs: &[&HostList])` | Ejecuta listas cerradas y espera el fence. |
| `pub(crate) fn log_messages(&self) -> Option<String>` | Vacía la cola del debug layer; primer error. |
| `pub(crate) fn rtv(&self, resource: &ID3D12Resource, desc: Option<&D3D12_RENDER_TARGET_VIEW_DESC>) -> D3D12_CPU_DESCRIPTOR_HANDLE` | RTV de un recurso (ring de 1024). |
| `pub(crate) fn cpu_view(&self) -> D3D12_CPU_DESCRIPTOR_HANDLE` | Slot de vista CPU-only. |
| `pub(crate) fn dsv(&self, resource: &ID3D12Resource, desc: Option<&D3D12_DEPTH_STENCIL_VIEW_DESC>) -> D3D12_CPU_DESCRIPTOR_HANDLE` | DSV de un recurso. |
| `pub(crate) fn default_buffer(&self, size: u64) -> Option<ID3D12Resource>` | Buffer por defecto (GPU-only). |
| `pub(crate) fn upload_buffer(&self, size: u64) -> Option<(ID3D12Resource, *mut u8)>` | Buffer de subida mapeado. |
| `pub(crate) fn present(&self, texture: &ID3D12Resource, state: u32, frame: u64) -> Option<bool>` | **Presenta** `texture` en la swapchain; `Some(black?)` en diagnostics. |
| `pub(crate) fn shared_buffer(&self, size: u64) -> Option<(ID3D12Resource, *mut u8)>` | Buffer compartido CPU/GPU. |
| `fn presenter(&self, width: u32, height: u32, format: DXGI_FORMAT) -> windows::core::Result<Presenter>` (privada) | **Crea la swapchain** con `CreateSwapChainForHwnd`. |

---

## 4. `impl HostList` (línea 998)

### Ciclo de vida
| Firma | Qué hace |
|---|---|
| `pub(crate) fn close(&self)` | Cierra la command list (y resuelve timestamps). |
| `pub(crate) fn reset(&self)` | Reinicia allocator/list y limpia el estado diferido. |

### Texturas / estados
| Firma | Qué hace |
|---|---|
| `pub(crate) fn transition(&self, resource: crate::object::This, wanted: u32) -> Option<(&'static ID3D12Resource, u32)>` | Estado deseado de una textura de título; devuelve textura y estado previo. |
| `pub(crate) fn move_texture(&self, copy: &Arc<crate::resource::HostTexture>, wanted: u32) -> Option<D3D12_RESOURCE_BARRIER>` | Barrier para una textura propia (twin). |
| `pub(crate) fn twin_use(&self, depth: &Arc<crate::resource::HostTexture>, side: u8, writes: bool) -> Option<bool>` | Uso de la cara twin de una profundidad. |
| `pub(crate) fn need_rebind(&self)` | Marca que hay que reenlazar root signature/pipeline. |
| `pub(crate) fn take_rebind(&self) -> bool` | Consume la marca anterior. |

### Diagnóstico
| Firma | Qué hace |
|---|---|
| `pub(crate) fn note_op(&self, what: impl FnOnce() -> String)` | Marca una operación. |
| `pub(crate) fn note_read(&self, what: impl FnOnce() -> String, range: (u64, u64))` | Marca una lectura con rango. |
| `pub(crate) fn note_target(&self, resource: usize, by: &str)` | Registra un render target. |
| `pub(crate) fn used_pipeline(&self, name: &str)` | Registra un pipeline usado. |

### Trabajo diferido a la submission
| Firma | Qué hace |
|---|---|
| `pub(crate) fn defer_copy(&self, dst: *mut u8, src: *const u8, len: usize)` | Copia diferida a memoria de título. |
| `pub(crate) fn defer_fill(&self, dst: *mut u8, src: *const u8, len: usize)` | Copia diferida a upload memory. |
| `pub(crate) fn defer_refresh(&self, shadow: &Arc<crate::shadow::Shadow>, address: usize, len: usize)` | Refresca un rango de copia host. |
| `pub(crate) fn defer_gpu_write(&self, shadow: &Arc<crate::shadow::Shadow>, address: usize, len: usize)` | Escritura GPU que vuelve a memoria de título. |
| `pub(crate) fn defer_heap(&self, viewed: &Arc<crate::bindless::Viewed>)` | Refresca un heap bindless. |
| `pub(crate) fn defer_memory(&self, heap: usize)` | Prepara page table de un heap. |
| `pub(crate) fn defer_missed(&self, check: impl Fn() -> bool + Send + 'static)` | Comprueba lecturas sin copia. |
| `pub(crate) fn defer_after(&self, work: impl Fn() + Send + 'static)` | Trabajo tras ejecutar la lista. |
| `pub(crate) fn defer_write(&self, dst: *mut u8, what: crate::queue::Written, bytes: usize)` | Escritura bottom-of-pipe. |
| `pub(crate) fn defer(&self, work: impl Fn() + Send + 'static)` | Trabajo en orden con las copias. |
| `pub(crate) fn defer_host(&self, work: impl Fn() + Send + 'static)` | Trabajo que solo toca memoria host. |
| `pub(crate) fn defer_texture(&self, staging: Arc<crate::texture::Staging>)` | Refresca el staging de una textura. |

### Memoria / objetos
| Firma | Qué hace |
|---|---|
| `pub(crate) fn upload(&self, host: &Host, size: u64) -> Option<(ID3D12Resource, *mut u8)>` | Upload buffer vivo hasta reset. |
| `pub(crate) fn scratch(&self, len: u64) -> Option<(u64, *mut u8)>` | Scratch de 256-byte alineado (GPU addr + CPU ptr). |
| `pub(crate) fn budget(&self) -> Option<u64>` | Dirección del presupuesto de trabajo indirecto. |
| `pub(crate) fn keep(&self, object: &impl Interface)` | Mantiene un objeto vivo hasta reset. |
| `pub(crate) fn first_use(&self, key: usize) -> bool` | Primer uso de una textura desde reset. |
| `fn prelude(&self) -> Option<ID3D12CommandList>` (privada) | Lista de transiciones previas. |
| `fn run_deferred(&self, batch: &mut Batch)` (privada) | Ejecuta los pasos diferidos. |

---

## 5. Puntos de entrada exactos

### ¿Dónde se crea el device?

`Host::host()` (línea 127) → `create()` (línea 139):

1. `D3D12GetDebugInterface` (diagnostics).
2. `D3D12CreateDevice(None, D3D_FEATURE_LEVEL_12_0, &mut device)` (o WARP
   con `VICTORY_D3D12_WARP`).
3. `CreateCommandQueue(D3D12_COMMAND_LIST_TYPE_DIRECT)`.
4. `CreateFence(0, ...)` + `CreateEventW`.
5. Tres descriptor heaps (RTV, DSV, CBV_SRV_UAV) de 1024.
6. `victory_shader::emit::host::Host { device }.fuses_fmad()`.
7. Devuelve `Host`.

### ¿Dónde se crea la swapchain?

`Host::present()` (línea 2035) → `Host::presenter()` (línea 2120):

1. `title_window()` (busca una ventana del proceso) o `window(width, height)`
   (crea una con `make_window` en su propio hilo).
2. `CreateDXGIFactory2(DXGI_CREATE_FACTORY_FLAGS(0))`.
3. `DXGI_SWAP_CHAIN_DESC1 { BufferCount: 2, SwapEffect: FLIP_DISCARD, ... }`.
4. `factory.CreateSwapChainForHwnd(&self.queue, hwnd, &desc, None, None)`
   → `IDXGISwapChain3`.
5. `self.new_list()` para la copia y `Presenter { ... }`.

### ¿Dónde se presenta?

`Host::present()` (línea 2035):

1. `swap_chain.GetCurrentBackBufferIndex()`.
2. `GetBuffer::<ID3D12Resource>(index)`.
3. Barriers textura→COPY_SOURCE, backbuffer PRESENT→COPY_DEST.
4. `list.CopyResource(&back, texture)`.
5. Barriers de vuelta.
6. `self.execute(&[&p.copy])`.
7. `p.swap_chain.Present(0, DXGI_PRESENT(0))`.
8. `p.meter.frame(p.hwnd)` (FPS + título de ventana).

---

## 6. Espejo Vulkan mínimo (`host_vulkan.rs`)

Correspondencia directa:

| D3D12 | Vulkan |
|---|---|
| `D3D12CreateDevice` | `vkCreateInstance` + `vkEnumeratePhysicalDevices` + `vkCreateDevice` |
| `ID3D12CommandQueue` (DIRECT) | `VkQueue` con `VK_QUEUE_GRAPHICS_BIT` |
| `IDXGISwapChain3` | `VkSwapchainKHR` |
| `HostList` (allocator + list) | `VkCommandPool` + `VkCommandBuffer` |
| `CopyResource` + `Present` | `vkCmdClearColorImage` / `vkCmdBeginRenderPass(clear)` + `vkQueuePresentKHR` |
| `ID3D12Fence` + event | `VkFence` + `vkWaitForFences` |
| `HWND` | surface X11/Wayland desde `raw-window-handle` |

Funciones que `host_vulkan.rs` debe exponer (mínimo):

```rust
pub fn create_instance() -> Result<Instance>;
pub fn create_device(instance: &Instance, surface: vk::SurfaceKHR)
    -> Result<(vk::PhysicalDevice, Device, vk::Queue, u32)>;
pub fn create_swapchain(device: &Device, physical: vk::PhysicalDevice,
    surface: vk::SurfaceKHR, width: u32, height: u32)
    -> Result<(vk::SwapchainKHR, Vec<vk::Image>, vk::Format, vk::Extent2D)>;
pub fn create_command_buffer(device: &Device, pool: vk::CommandPool)
    -> Result<Vec<vk::CommandBuffer>>;
pub fn present_clear(/* swapchain, images, command buffers, queue, ... */)
    -> Result<()>;
```

---

## 7. Conclusión de la auditoría

- El host D3D12 está **fuertemente acoplado** a: `ID3D12Device`, cola
  DIRECT, fence, tres heaps de descriptores, `HostList` con trabajo diferido
  y `Presenter` con `IDXGISwapChain3`.
- La superficie mínima para un espejo Vulkan es pequeña: instancia,
  physical device, device + cola, surface, swapchain, command pool/buffers,
  render pass con `LOAD_OP_CLEAR`, y `vkQueuePresentKHR`.
- El resto de `host.rs` (transiciones, twins, bindless, shadow copies,
  capturas, métricas) **no es necesario** para el clear color y se puede
  dejar fuera de `host_vulkan.rs` en esta fase.
