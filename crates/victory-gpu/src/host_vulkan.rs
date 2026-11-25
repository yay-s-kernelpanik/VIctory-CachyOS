//! Minimal Vulkan host for Linux/CachyOS: instance -> device -> swapchain ->
//! command buffers -> clear + present.
//!
//! The D3D12 host lives in `host.rs` (Windows). This module is the Linux
//! counterpart for the smoke path: open a window, clear it to a colour and
//! present one frame. It does not load titles, shaders or assets.

use anyhow::{Context, Result};
use ash::vk;
use ash_window::enumerate_required_extensions;
use raw_window_handle::{HasDisplayHandle, RawDisplayHandle, RawWindowHandle};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

/// The clear colour: a visible blue.
const CLEAR: [f32; 4] = [0.1, 0.2, 0.6, 1.0];

/// A trivial compute shader (`void main() {}`, 1x1x1), compiled with
/// glslangValidator from GLSL 450. It proves the SPIR-V -> VkShaderModule ->
/// VkComputePipeline -> vkCmdDispatch path without game data.
const COMPUTE_SPV: &[u32] = &[
    0x07230203, 0x00010000, 0x0008000b, 0x0000000a, 0x00000000, 0x00020011, 0x00000001, 0x0006000b,
    0x00000001, 0x4c534c47, 0x6474732e, 0x3035342e, 0x00000000, 0x0003000e, 0x00000000, 0x00000001,
    0x0005000f, 0x00000005, 0x00000004, 0x6e69616d, 0x00000000, 0x00060010, 0x00000004, 0x00000011,
    0x00000001, 0x00000001, 0x00000001, 0x00030003, 0x00000002, 0x000001c2, 0x00040005, 0x00000004,
    0x6e69616d, 0x00000000, 0x00040047, 0x00000009, 0x0000000b, 0x00000019, 0x00020013, 0x00000002,
    0x00030021, 0x00000003, 0x00000002, 0x00040015, 0x00000006, 0x00000020, 0x00000000, 0x00040017,
    0x00000007, 0x00000006, 0x00000003, 0x0004002b, 0x00000006, 0x00000008, 0x00000001, 0x0006002c,
    0x00000007, 0x00000009, 0x00000008, 0x00000008, 0x00000008, 0x00050036, 0x00000002, 0x00000004,
    0x00000000, 0x00000003, 0x000200f8, 0x00000005, 0x000100fd, 0x00010038,
];

/// All the Vulkan state of the smoke path.
pub struct Vulkan {
    pub entry: Option<ash::Entry>,
    pub instance: Option<ash::Instance>,
    pub physical: Option<vk::PhysicalDevice>,
    pub device: Option<ash::Device>,
    pub queue: Option<vk::Queue>,
    pub queue_family: u32,
    pub surface_loader: Option<ash::khr::surface::Instance>,
    pub surface: Option<vk::SurfaceKHR>,
    pub swapchain_loader: Option<ash::khr::swapchain::Device>,
    pub swapchain: Option<vk::SwapchainKHR>,
    pub images: Vec<vk::Image>,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub command_pool: Option<vk::CommandPool>,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub device_name: String,
    pub device_type: vk::PhysicalDeviceType,
    pub api_version: u32,
    pub driver_version: u32,
    pub compute_module: Option<vk::ShaderModule>,
    pub compute_layout: Option<vk::PipelineLayout>,
    pub compute_pipeline: Option<vk::Pipeline>,
}

impl Default for Vulkan {
    fn default() -> Self {
        Self {
            entry: None,
            instance: None,
            physical: None,
            device: None,
            queue: None,
            queue_family: 0,
            surface_loader: None,
            surface: None,
            swapchain_loader: None,
            swapchain: None,
            images: Vec::new(),
            format: vk::Format::UNDEFINED,
            extent: vk::Extent2D::default(),
            command_pool: None,
            command_buffers: Vec::new(),
            device_name: String::new(),
            device_type: vk::PhysicalDeviceType::OTHER,
            api_version: 0,
            driver_version: 0,
            compute_module: None,
            compute_layout: None,
            compute_pipeline: None,
        }
    }
}

impl Vulkan {
    /// `vkCreateInstance` with the surface extensions the display needs.
    pub fn create_instance(&mut self, display: RawDisplayHandle) -> Result<()> {
        let entry = unsafe { ash::Entry::load().context("loading libvulkan")? };
        let required = enumerate_required_extensions(display).context("surface extensions")?;
        let app_info = vk::ApplicationInfo::default()
            .application_name(c"VIctory")
            .api_version(vk::make_api_version(0, 1, 0, 0));
        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(required);
        let instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .context("vkCreateInstance")?
        };
        self.entry = Some(entry);
        self.instance = Some(instance);
        Ok(())
    }

    /// Picks a graphics device, creates the logical device and the surface.
    pub fn create_device(
        &mut self,
        display: RawDisplayHandle,
        window: RawWindowHandle,
    ) -> Result<()> {
        let entry = self.entry.as_ref().context("no entry")?;
        let instance = self.instance.as_ref().context("no instance")?;
        let surface = unsafe {
            ash_window::create_surface(entry, instance, display, window, None)
                .context("vkCreateSurface")?
        };
        let surface_loader = ash::khr::surface::Instance::new(entry, instance);
        let physical = unsafe { instance.enumerate_physical_devices()? }
            .into_iter()
            .next()
            .context("no Vulkan physical device")?;
        let queue_family = unsafe {
            instance
                .get_physical_device_queue_family_properties(physical)
                .iter()
                .enumerate()
                .find(|(_, q)| q.queue_flags.contains(vk::QueueFlags::GRAPHICS))
                .context("no graphics queue")?
                .0 as u32
        };
        let priorities = [1.0f32];
        let queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family)
            .queue_priorities(&priorities);
        let extensions = [ash::khr::swapchain::NAME.as_ptr()];
        let device_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&extensions);
        let device = unsafe { instance.create_device(physical, &device_info, None)? };
        let queue = unsafe { device.get_device_queue(queue_family, 0) };
        let swapchain_loader = ash::khr::swapchain::Device::new(instance, &device);
        let props = unsafe { instance.get_physical_device_properties(physical) };

        self.device_name = props
            .device_name_as_c_str()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "unknown".to_owned());
        self.device_type = props.device_type;
        self.api_version = props.api_version;
        self.driver_version = props.driver_version;
        self.physical = Some(physical);
        self.device = Some(device);
        self.queue = Some(queue);
        self.queue_family = queue_family;
        self.surface_loader = Some(surface_loader);
        self.surface = Some(surface);
        self.swapchain_loader = Some(swapchain_loader);
        Ok(())
    }

    /// Creates a trivial compute pipeline from embedded SPIR-V.
    pub fn create_compute_pipeline(&mut self) -> Result<()> {
        let device = self.device.as_ref().context("no device")?;
        let module_info = vk::ShaderModuleCreateInfo::default().code(COMPUTE_SPV);
        let module = unsafe { device.create_shader_module(&module_info, None)? };
        let layout_info = vk::PipelineLayoutCreateInfo::default();
        let layout = unsafe { device.create_pipeline_layout(&layout_info, None)? };
        let stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(module)
            .name(c"main");
        let info = vk::ComputePipelineCreateInfo::default()
            .stage(stage)
            .layout(layout);
        let pipeline = unsafe {
            device
                .create_compute_pipelines(vk::PipelineCache::null(), &[info], None)
                .map_err(|(_, e)| e)?[0]
        };
        self.compute_module = Some(module);
        self.compute_layout = Some(layout);
        self.compute_pipeline = Some(pipeline);
        Ok(())
    }

    /// `vkCreateSwapchainKHR` with the surface's format and extent.
    pub fn create_swapchain(&mut self, width: u32, height: u32) -> Result<()> {
        let instance = self.instance.as_ref().context("no instance")?;
        let physical = self.physical.context("no physical device")?;
        let device = self.device.as_ref().context("no device")?;
        let surface = self.surface.context("no surface")?;
        let surface_loader = self.surface_loader.as_ref().context("no surface loader")?;
        let swapchain_loader = self
            .swapchain_loader
            .as_ref()
            .context("no swapchain loader")?;

        let formats =
            unsafe { surface_loader.get_physical_device_surface_formats(physical, surface)? };
        let format = formats
            .iter()
            .find(|f| f.format == vk::Format::B8G8R8A8_SRGB)
            .copied()
            .or_else(|| formats.first().copied())
            .context("no surface format")?;
        let present_modes =
            unsafe { surface_loader.get_physical_device_surface_present_modes(physical, surface)? };
        let present_mode = present_modes
            .iter()
            .copied()
            .find(|m| *m == vk::PresentModeKHR::FIFO)
            .or_else(|| present_modes.first().copied())
            .context("no present mode")?;
        let caps =
            unsafe { surface_loader.get_physical_device_surface_capabilities(physical, surface)? };
        let extent = if caps.current_extent.width == 0
            || caps.current_extent.height == 0
            || caps.current_extent.width == u32::MAX
        {
            vk::Extent2D {
                width: width.max(1),
                height: height.max(1),
            }
        } else {
            caps.current_extent
        };

        let info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(caps.min_image_count.max(2))
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(caps.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);
        let swapchain = unsafe { swapchain_loader.create_swapchain(&info, None)? };
        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };

        // keep `instance` used so the borrow above stays explicit
        let _ = instance;
        let _ = device;

        self.format = format.format;
        self.extent = extent;
        self.swapchain = Some(swapchain);
        self.images = images;
        Ok(())
    }

    /// `vkAllocateCommandBuffers`, one per swapchain image, recording a clear.
    pub fn create_command_buffer(&mut self) -> Result<()> {
        let device = self.device.as_ref().context("no device")?;
        let pool_info = vk::CommandPoolCreateInfo::default().queue_family_index(self.queue_family);
        let pool = unsafe { device.create_command_pool(&pool_info, None)? };
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(self.images.len() as u32);
        let buffers = unsafe { device.allocate_command_buffers(&alloc_info)? };

        let range = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        };
        let clear = vk::ClearColorValue { float32: CLEAR };

        for (i, cb) in buffers.iter().enumerate() {
            let begin = vk::CommandBufferBeginInfo::default();
            unsafe { device.begin_command_buffer(*cb, &begin)? };

            // UNDEFINED -> TRANSFER_DST_OPTIMAL
            let to_dst = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(self.images[i])
                .subresource_range(range)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);
            unsafe {
                device.cmd_pipeline_barrier(
                    *cb,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[to_dst],
                );
                device.cmd_clear_color_image(
                    *cb,
                    self.images[i],
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &clear,
                    &[range],
                );
                if let Some(pipeline) = self.compute_pipeline {
                    device.cmd_bind_pipeline(*cb, vk::PipelineBindPoint::COMPUTE, pipeline);
                    device.cmd_dispatch(*cb, 1, 1, 1);
                }
            }

            // TRANSFER_DST_OPTIMAL -> PRESENT_SRC_KHR
            let to_present = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(self.images[i])
                .subresource_range(range)
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::empty());
            unsafe {
                device.cmd_pipeline_barrier(
                    *cb,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[to_present],
                );
                device.end_command_buffer(*cb)?;
            }
        }

        self.command_pool = Some(pool);
        self.command_buffers = buffers;
        Ok(())
    }

    /// Acquires an image, submits its clear and presents it.
    pub fn present_clear(&mut self) -> Result<()> {
        let device = self.device.as_ref().context("no device")?;
        let queue = self.queue.context("no queue")?;
        let swapchain = self.swapchain.context("no swapchain")?;
        let loader = self
            .swapchain_loader
            .as_ref()
            .context("no swapchain loader")?;

        let (index, _) = unsafe {
            loader.acquire_next_image(
                swapchain,
                u64::MAX,
                vk::Semaphore::null(),
                vk::Fence::null(),
            )?
        };
        let cb = *self
            .command_buffers
            .get(index as usize)
            .context("no command buffer for image")?;
        let submit = vk::SubmitInfo::default().command_buffers(std::slice::from_ref(&cb));
        unsafe { device.queue_submit(queue, &[submit], vk::Fence::null())? };
        let present = vk::PresentInfoKHR::default()
            .swapchains(std::slice::from_ref(&swapchain))
            .image_indices(std::slice::from_ref(&index));
        unsafe { loader.queue_present(queue, &present)? };
        unsafe { device.device_wait_idle()? };
        tracing::info!("presented one clear-colour frame");
        Ok(())
    }
}

struct App {
    window: Option<Arc<dyn Window>>,
    vulkan: Vulkan,
    presented: bool,
    report: bool,
}

impl ApplicationHandler for App {
    fn new_events(&mut self, event_loop: &dyn ActiveEventLoop, cause: StartCause) {
        if matches!(cause, StartCause::Init) && self.window.is_none() {
            self.open(event_loop);
        }
    }

    fn resumed(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.window.is_none() {
            self.open(event_loop);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if !self.presented {
                    match self.vulkan.present_clear() {
                        Ok(()) if self.report => println!("GPU check: PASS"),
                        Ok(()) => {}
                        Err(e) => {
                            if self.report {
                                println!("GPU check: FAIL: {e}");
                            }
                            tracing::error!(error = %e, "present_clear failed");
                        }
                    }
                    self.presented = true;
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
                event_loop.exit();
            }
            _ => {}
        }
    }

    fn can_create_surfaces(&mut self, _event_loop: &dyn ActiveEventLoop) {}
}

impl App {
    fn open(&mut self, event_loop: &dyn ActiveEventLoop) {
        let attrs = WindowAttributes::default().with_title("VIctory Vulkan");
        let window = event_loop.create_window(attrs).expect("create window");
        let display = window
            .as_ref()
            .display_handle()
            .expect("display handle")
            .as_raw();
        let raw = window
            .as_ref()
            .rwh_06_window_handle()
            .window_handle()
            .expect("window handle")
            .as_raw();
        self.vulkan
            .create_instance(display)
            .expect("create_instance");
        self.vulkan
            .create_device(display, raw)
            .expect("create_device");
        self.vulkan
            .create_swapchain(800, 600)
            .expect("create_swapchain");
        self.vulkan
            .create_compute_pipeline()
            .expect("create_compute_pipeline");
        self.vulkan
            .create_command_buffer()
            .expect("create_command_buffer");
        if self.report {
            let api = self.vulkan.api_version;
            println!("GPU check");
            println!(
                "  device:    {} ({:?})",
                self.vulkan.device_name, self.vulkan.device_type
            );
            println!(
                "  vulkan:    {}.{}.{}",
                vk::api_version_major(api),
                vk::api_version_minor(api),
                vk::api_version_patch(api)
            );
            println!("  driver:    {:#x}", self.vulkan.driver_version);
            println!(
                "  queue:     family {} (graphics)",
                self.vulkan.queue_family
            );
            println!("  compute:   dispatch 1x1x1");
            println!(
                "  swapchain: {} images, {}x{}, {:?}",
                self.vulkan.images.len(),
                self.vulkan.extent.width,
                self.vulkan.extent.height,
                self.vulkan.format
            );
        }
        tracing::info!(
            "Vulkan ready: {}x{}, format {:?}",
            self.vulkan.extent.width,
            self.vulkan.extent.height,
            self.vulkan.format
        );
        self.window = Some(Arc::from(window));
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}

fn run_inner(report: bool) -> Result<()> {
    let event_loop = EventLoop::new().context("creating event loop")?;
    let app = App {
        window: None,
        vulkan: Vulkan::default(),
        presented: false,
        report,
    };
    event_loop.run_app(app).context("running event loop")?;
    Ok(())
}

/// Opens a window, initialises Vulkan and presents one clear-colour frame.
pub fn run() -> Result<()> {
    run_inner(false)
}

/// Hardware/runtime check without game data: device, surface, swapchain and
/// one clear + present. Prints a PASS/FAIL report.
pub fn gpu_check() -> Result<()> {
    run_inner(true)
}
