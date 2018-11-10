use ash::version::{EntryV1_0, InstanceV1_0, V1_1};
use ash::vk;

use std::ffi::CString;
use std::ptr;

pub mod device;
pub mod swapchain;

const LAYERS: &[*const i8] = &[];

const EXTENSION: &[*const i8] = &[
    b"VK_KHR_surface\0".as_ptr() as *const _,
    b"VK_KHR_win32_surface\0".as_ptr() as *const _,
];

pub struct Adapter {
    physical_device: vk::PhysicalDevice,
    queue_families: Vec<vk::QueueFamilyProperties>,
    properties: vk::PhysicalDeviceProperties,
    features: vk::PhysicalDeviceFeatures,
}

pub struct Engine {
    entry: ash::Entry<V1_1>,
    instance: ash::Instance<V1_1>,
    pub(crate) surface_win32: ash::extensions::Win32Surface,
}

impl Engine {
    pub fn new() -> Self {
        let entry: ash::Entry<V1_1> = ash::Entry::new().unwrap();

        let app_name = CString::new("tanya").unwrap();
        let engine_name = CString::new("tanya").unwrap();

        let app_info = vk::ApplicationInfo {
            p_application_name: app_name.as_ptr(),
            s_type: vk::StructureType::APPLICATION_INFO,
            p_next: ptr::null(),
            application_version: 0,
            p_engine_name: engine_name.as_ptr(),
            engine_version: 0,
            api_version: ash::vk_make_version!(1, 1, 0),
        };

        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            pp_enabled_layer_names: LAYERS.as_ptr(),
            enabled_layer_count: LAYERS.len() as u32,
            pp_enabled_extension_names: EXTENSION.as_ptr(),
            enabled_extension_count: EXTENSION.len() as u32,
        };

        let instance: ash::Instance<V1_1> = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Couldn't create instance")
        };

        let surface_win32 = ash::extensions::Win32Surface::new(&entry, &instance).unwrap();

        Engine {
            entry,
            instance,
            surface_win32,
        }
    }

    pub fn enumerate_adapters(&self) -> Vec<Adapter> {
        self.instance
            .enumerate_physical_devices()
            .unwrap()
            .iter()
            .map(|physical_device| {
                let queue_families = self
                    .instance
                    .get_physical_device_queue_family_properties(*physical_device);

                Adapter {
                    physical_device,
                    queue_families,
                }
            })
            .collect()
    }
}
