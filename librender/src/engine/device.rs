use super::{Adapter, Engine};
use ash::version::{InstanceV1_0, V1_1};
use ash::vk;
use std::ptr;

const EXTENSION: &[*const i8] = &[b"VK_KHR_swapchain\0".as_ptr() as *const _];

pub struct Device {
    device: ash::Device<V1_1>,
    pub(crate) swapchain: ash::extensions::Swapchain,
}

impl Engine {
    pub fn create_device(&self, adapter: &Adapter) -> Device {
        let queue_info = vk::DeviceQueueCreateInfo {};

        let features = vk::PhysicalDeviceFeatures {
            ..Default::default()
        };

        let create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: 1,
            p_queue_create_infos: &queue_info,
            enabled_layer_count: 0,
            pp_enabled_layer_names: ptr::null(),
            enabled_extension_count: EXTENSION.len() as u32,
            pp_enabled_extension_names: EXTENSION.as_ptr(),
            p_enabled_features: &features,
        };

        let device: ash::Device<V1_1> = self
            .instance
            .create_device(adapter.physical_device, &create_info, None)
            .unwrap();

        let swapchain = ash::extensions::Swapchain::new(&self.instance, &device).unwrap();

        Device { swapchain }
    }
}
