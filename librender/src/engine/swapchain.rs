use ash::vk;
use crate::{
    display::Display,
    engine::{device::Device, Engine},
};
use std::ptr;

pub struct Swapchain {}

#[derive(Debug, Copy, Clone)]
pub struct Config {
    min_image_count: usize,
    width: u32,
    height: u32,
    color_space: vk::ColorSpaceKHR,
    present_mode: vk::PresentModeKHR,
    format: vk::Format,
}

impl Engine {
    pub fn create_swapchain<D: Display>(
        &self,
        device: &Device,
        display: &D,
        config: Config,
    ) -> Swapchain {
        let extent = vk::Extent2D {
            width: config.width,
            height: config.height,
        };
        let create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: vk::SwapchainCreateFlagsKHR::empty(),
            surface: display.surface(),
            min_image_count: config.min_image_count as _,
            image_color_space: config.color_space,
            image_format: config.format,
            image_extent: extent,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            pre_transform: vk::SurfaceTransformFlagsKHR::IDENTITY,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode: config.present_mode,
            clipped: 1,
            old_swapchain: vk::SwapchainKHR::null(),
            image_array_layers: 1,
            p_queue_family_indices: ptr::null(),
            queue_family_index_count: 0,
        };

        let swapchain = unsafe {
            device
                .swapchain
                .create_swapchain_khr(&create_info, None)
                .unwrap()
        };

        Swapchain {}
    }
}
