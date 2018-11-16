use super::{Display, Surface};
use ash::{extensions, vk};
use crate::Engine;
use std::ptr;

pub struct WindowDisplay {
    surface: vk::SurfaceKHR,
    surface_fn: extensions::Surface,
}

impl WindowDisplay {
    pub fn new(engine: &Engine, window: &winit::Window) -> Self {
        use winapi::um::libloaderapi::GetModuleHandleW;
        use winit::os::windows::WindowExt;

        let hwnd = window.get_hwnd();
        let hinstance = unsafe { GetModuleHandleW(ptr::null()) as *const _ };
        let create_info = vk::Win32SurfaceCreateInfoKHR {
            s_type: vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: vk::Win32SurfaceCreateFlagsKHR::empty(),
            hinstance: hinstance,
            hwnd: hwnd as *const _,
        };
        let surface = unsafe {
            engine
                .surface_win32
                .create_win32_surface_khr(&create_info, None)
                .unwrap()
        };
        let surface_fn = extensions::Surface::new(&engine.entry, &engine.instance);

        WindowDisplay {
            surface,
            surface_fn,
        }
    }
}

impl Display for WindowDisplay {
    fn surface(&self) -> Surface {
        Surface {
            surface: &self.surface,
            surface_fn: &self.surface_fn,
        }
    }
}
