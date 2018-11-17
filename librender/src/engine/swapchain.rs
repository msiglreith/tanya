use ash::{extensions, vk};
use crate::{
    display::Display,
    engine::{device::Device, Engine},
};
use std::ptr;

pub type Frame = usize;

pub struct Swapchain {
    swapchain: vk::SwapchainKHR,
    swapchain_fn: extensions::Swapchain,
    images: Vec<vk::Image>,
}

impl Swapchain {
    pub fn begin_frame(&self, semaphore: vk::Semaphore) -> Frame {
        let index = unsafe {
            self.swapchain_fn.acquire_next_image_khr(
                self.swapchain,
                !0,
                semaphore,
                vk::Fence::null(),
            )
        };

        index.map(|(i, _)| i as Frame).unwrap()
    }

    pub fn get_image(&self, frame: &Frame) -> vk::Image {
        self.images[*frame]
    }

    pub fn end_frame(&self, frame: Frame, queue: vk::Queue, wait_semaphores: &[vk::Semaphore]) {
        let mut result = vk::Result::SUCCESS;
        let frame_u32 = frame as u32;
        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: ptr::null(),
            wait_semaphore_count: wait_semaphores.len() as _,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: &self.swapchain as *const _,
            p_image_indices: &frame_u32 as *const _,
            p_results: &mut result as *mut _,
        };
        unsafe { self.swapchain_fn.queue_present_khr(queue, &present_info) };
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Config {
    pub min_image_count: usize,
    pub width: u32,
    pub height: u32,
    pub color_space: vk::ColorSpaceKHR,
    pub present_mode: vk::PresentModeKHR,
    pub format: vk::Format,
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
            surface: display.surface().raw(),
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

        let swapchain_fn = extensions::Swapchain::new(&self.instance, &device.device);
        let images = unsafe { swapchain_fn.get_swapchain_images_khr(swapchain).unwrap() };

        Swapchain {
            swapchain,
            swapchain_fn,
            images,
        }
    }
}
