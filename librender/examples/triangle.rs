extern crate tanya_render as render;

use self::render::display::Display;
use self::render::vk;

use failure::Error;
use winit::{dpi::LogicalSize, WindowEvent};

const NUM_FRAMES: usize = 2;

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
    let adapter = &adapters[0];
    let display = render::display::WindowDisplay::new(&engine, &window);
    println!("{:#?}", adapters);

    let main_queue_family = 0;
    assert!(
        display
            .surface()
            .adapter_supported(adapter, main_queue_family)
    );
    let main_queue_info = render::engine::device::QueueCreateInfo {
        family: main_queue_family,
        queues: vec![1.0f32],
    };
    let device = engine.create_device(adapter, &[main_queue_info]);
    let main_queue = device.get_queue(main_queue_family, 0);

    let swapchain_config = render::swapchain::Config {
        min_image_count: NUM_FRAMES as _,
        width,
        height,
        color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
        present_mode: vk::PresentModeKHR::FIFO,
        format: vk::Format::R8G8B8A8_SRGB,
    };
    let swapchain = engine.create_swapchain(&device, &display, swapchain_config);

    let fences: [vk::Fence; NUM_FRAMES] = [device.create_fence(false), device.create_fence(false)];
    let frame_ready: [vk::Semaphore; NUM_FRAMES] =
        [device.create_semaphore(), device.create_semaphore()];

    let mut quit = false;
    let mut tick = 0;

    loop {
        events_loop.poll_events(|event| match event {
            winit::Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => quit = true,
            _ => {}
        });

        let frame = swapchain.begin_frame(frame_ready[tick % NUM_FRAMES]);

        if quit {
            break;
        }

        swapchain.end_frame(frame, main_queue);

        tick += 1;
    }

    device.destroy();

    Ok(())
}
