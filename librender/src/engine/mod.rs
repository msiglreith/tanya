use d3d12::{self, dxgi};

pub struct Engine {
    pub factory: dxgi::Factory4,
}

impl Engine {
    pub fn new(debug: bool) -> Self {
        if debug {
            Engine::init_debug();
        }

        let mut factory_flags = dxgi::FactoryCreationFlags::empty();
        if debug {
            factory_flags |= dxgi::FactoryCreationFlags::DEBUG;
        }
        let (factory, _) = dxgi::Factory4::create(factory_flags);

        Engine { factory }
    }

    fn init_debug() {
        // Enable debug
        let (debug, _) = d3d12::Debug::get_debug_interface();
        debug.enable_debug_layer();
        unsafe {
            debug.destroy();
        }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe {
            self.factory.destroy();
        }
    }
}
