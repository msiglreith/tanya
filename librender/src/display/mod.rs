use ash::{extensions, vk};

use crate::engine::Adapter;

pub mod window;

pub use self::window::WindowDisplay;

pub struct Surface<'a> {
    surface: &'a vk::SurfaceKHR,
    surface_fn: &'a extensions::Surface,
}

impl<'a> Surface<'a> {
    pub fn raw(&self) -> vk::SurfaceKHR {
        *self.surface
    }

    pub fn adapter_supported(&self, adapter: &Adapter, family: usize) -> bool {
        unsafe {
            self.surface_fn.get_physical_device_surface_support_khr(
                adapter.physical_device,
                family as _,
                *self.surface,
            )
        }
    }

    pub fn query_support(&self, adapter: &Adapter) -> SurfaceSupport {
        let capabilities = unsafe {
            self.surface_fn
                .get_physical_device_surface_capabilities_khr(
                    adapter.physical_device,
                    *self.surface,
                )
                .unwrap()
        };

        let present_modes = unsafe {
            self.surface_fn
                .get_physical_device_surface_present_modes_khr(
                    adapter.physical_device,
                    *self.surface,
                )
                .unwrap()
        };

        let formats = unsafe {
            self.surface_fn
                .get_physical_device_surface_formats_khr(adapter.physical_device, *self.surface)
                .unwrap()
        };

        SurfaceSupport {
            capabilities,
            present_modes,
            formats,
        }
    }
}

#[derive(Debug)]
pub struct SurfaceSupport {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub present_modes: Vec<vk::PresentModeKHR>,
    pub formats: Vec<vk::SurfaceFormatKHR>,
}

pub trait Display {
    fn surface(&self) -> Surface;
}
