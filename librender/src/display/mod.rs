use ash::vk;

pub mod window;

pub use self::window::WindowDisplay;

pub struct Surface<'a> {
    raw: &'a vk::SurfaceKHR,
}

impl<'a> Surface<'a> {
    pub fn raw(&self) -> vk::SurfaceKHR {
        *self.raw
    }
}

pub trait Display {
    fn surface(&self) -> Surface;
}
