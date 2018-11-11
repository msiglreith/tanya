extern crate tanya_render as render;

use self::render::display::Display;
use self::render::vk;

use failure::Error;
use winit::{dpi::LogicalSize, WindowEvent};

fn main() -> Result<(), Error> {
    let (width, height) = (1440, 720);

    let mut events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new()
        .with_dimensions(LogicalSize {
            width: width as _,
            height: height as _,
        })
        .with_title("tanya - hati sample")
        .build(&events_loop)?;

    let engine = render::Engine::new();
    let adapters = engine.enumerate_adapters();
    println!("{:#?}", adapters);

    let device = engine.create_device(&adapters[0]);
    let display = render::display::WindowDisplay::new(&engine, &window);
    let swapchain_config = render::swapchain::Config {
        min_image_count: 2,
        width,
        height,
        color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
        present_mode: vk::PresentModeKHR::FIFO,
        format: vk::Format::R8G8B8A8_SRGB,
    };
    let swapchain = engine.create_swapchain(&device, &display, swapchain_config);

    let mut quit = false;
    loop {
        events_loop.poll_events(|event| match event {
            winit::Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => quit = true,
            _ => {}
        });

        if quit {
            break;
        }
    }

    device.destroy();

    Ok(())
}
