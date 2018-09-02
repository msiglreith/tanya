use failure::Error;
use winit::{dpi::LogicalSize, WindowEvent};

fn main() -> Result<(), Error> {
    let mut events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new()
        .with_dimensions(LogicalSize {
            width: 1440.0,
            height: 720.0,
        }).with_title("tanya - hati sample")
        .build(&events_loop)?;

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
