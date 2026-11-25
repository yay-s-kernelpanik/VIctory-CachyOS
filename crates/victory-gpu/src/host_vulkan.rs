//! Minimal Vulkan host for Linux/CachyOS: instance and device.
//!
//! The D3D12 host lives in `host.rs` (Windows). This module is the Linux
//! counterpart; the swapchain and present path land in the next commit.

use anyhow::{Context, Result};
use ash::vk;
use ash_window::enumerate_required_extensions;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

/// The Vulkan state created by the smoke path.
pub struct Vulkan {
    pub entry: Option<ash::Entry>,
    pub instance: Option<ash::Instance>,
    pub physical: Option<vk::PhysicalDevice>,
    pub device: Option<ash::Device>,
    pub queue: Option<vk::Queue>,
    pub queue_family: u32,
    pub surface_loader: Option<ash::khr::surface::Instance>,
    pub surface: Option<vk::SurfaceKHR>,
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

        self.physical = Some(physical);
        self.device = Some(device);
        self.queue = Some(queue);
        self.queue_family = queue_family;
        self.surface_loader = Some(surface_loader);
        self.surface = Some(surface);
        Ok(())
    }
}

struct App {
    window: Option<Arc<dyn Window>>,
    vulkan: Vulkan,
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

    fn window_event(&mut self, event_loop: &dyn ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
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
        tracing::info!("Vulkan instance and device ready");
        self.window = Some(Arc::from(window));
        event_loop.exit();
    }
}

/// Opens a window and initialises the Vulkan instance and device.
pub fn run() -> Result<()> {
    let event_loop = EventLoop::new().context("creating event loop")?;
    let app = App {
        window: None,
        vulkan: Vulkan::default(),
    };
    event_loop.run_app(app).context("running event loop")?;
    Ok(())
}
