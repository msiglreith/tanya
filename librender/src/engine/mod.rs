use d3d12::{self, dxgi};
use winapi::shared::winerror;

pub mod swap_chain;

pub struct Engine {
    pub factory: dxgi::Factory4,
    pub device: d3d12::Device,
    pub queue: d3d12::CommandQueue,
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
        let device = Engine::select_adapter(factory);

        let (queue, _) = device.create_command_queue(
            d3d12::command_list::CmdListType::Direct,
            d3d12::queue::Priority::Normal,
            d3d12::queue::CommandQueueFlags::empty(),
            0,
        );

        Engine {
            factory,
            device,
            queue,
        }
    }

    fn select_adapter(factory: dxgi::Factory4) -> d3d12::Device {
        let mut id = 0;
        loop {
            let (adapter, hr) = factory.enumerate_adapters(id);
            if hr == winerror::DXGI_ERROR_NOT_FOUND {
                panic!("unable to find adapter")
            }
            id += 1;

            // Check for D3D12 support
            {
                let (device, hr) = d3d12::Device::create(adapter, d3d12::FeatureLevel::L12_0);
                if !winerror::SUCCEEDED(hr) {
                    continue;
                }
                unsafe { adapter.destroy() };
                return device;
            };
        }
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
            self.device.destroy();
            self.queue.destroy();
            self.factory.destroy();
        }
    }
}
