extern crate tanya_render2 as render;

use self::render::display::Display;

use failure::Error;
use winit::{dpi::LogicalSize, WindowEvent};

fn main() -> Result<(), Error> {
    let (width, height) = (1440, 720);

    let mut events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new()
        .with_dimensions(LogicalSize {
            width: width as _,
            height: height as _,
        }).with_title("tanya - hati sample")
        .build(&events_loop)?;

    let engine = render::Engine::new();

    let display = render::display::WindowDisplay::new(&engine, &window);
    let swapchain = engine.create_swapchain(&display);

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

    Ok(())
}
