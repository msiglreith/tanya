use super::{Adapter, Engine};
use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;
use std::ptr;

const EXTENSION: &[*const i8] = &[b"VK_KHR_swapchain\0".as_ptr() as *const _];

pub struct Device {
    pub(crate) device: ash::Device,
    pub(crate) swapchain: ash::extensions::Swapchain,
}

impl Device {
    pub fn get_queue(&self, family: usize, index: u32) -> vk::Queue {
        unsafe { self.device.get_device_queue(family as _, index) }
    }

    pub fn create_fence(&self, signaled: bool) -> vk::Fence {
        let flags = if signaled {
            vk::FenceCreateFlags::SIGNALED
        } else {
            vk::FenceCreateFlags::empty()
        };
        let create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags,
        };

        unsafe { self.device.create_fence(&create_info, None).unwrap() }
    }

    pub fn create_semaphore(&self) -> vk::Semaphore {
        let create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SemaphoreCreateFlags::empty(),
        };
        let semaphore = unsafe { self.device.create_semaphore(&create_info, None).unwrap() };
        println!("{:?}", (semaphore));
        semaphore
    }

    pub fn destroy(self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }

    pub fn destroy_fence(&self, fence: vk::Fence) {
        unsafe {
            self.device.destroy_fence(fence, None);
        }
    }
}

pub type QueuePriority = f32;

pub struct QueueCreateInfo {
    pub family: usize,
    pub queues: Vec<QueuePriority>,
}

impl Engine {
    pub fn create_device(&self, adapter: &Adapter, queues: &[QueueCreateInfo]) -> Device {
        let queue_infos = queues
            .iter()
            .map(|q| vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::DeviceQueueCreateFlags::empty(),
                queue_family_index: q.family as _,
                queue_count: q.queues.len() as _,
                p_queue_priorities: q.queues.as_ptr(),
            })
            .collect::<Vec<_>>();

        let features = vk::PhysicalDeviceFeatures {
            ..Default::default()
        };

        let create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: queue_infos.len() as _,
            p_queue_create_infos: queue_infos.as_ptr(),
            enabled_layer_count: 0,
            pp_enabled_layer_names: ptr::null(),
            enabled_extension_count: EXTENSION.len() as u32,
            pp_enabled_extension_names: EXTENSION.as_ptr(),
            p_enabled_features: &features,
        };

        let device: ash::Device = unsafe {
            self.instance
                .create_device(adapter.physical_device, &create_info, None)
                .unwrap()
        };

        let swapchain = ash::extensions::Swapchain::new(&self.instance, &device);

        Device { device, swapchain }
    }
}
