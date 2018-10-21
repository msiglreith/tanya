use ash::vk;

pub mod window;

pub use self::window::WindowDisplay;

pub trait Display {
    fn surface(&self) -> vk::SurfaceKHR;
}
