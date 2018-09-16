use failure::Error;
use tanya::render;
use winit::{dpi::LogicalSize, WindowEvent};

const BUFFER_COUNT: u32 = 2;

fn main() -> Result<(), Error> {
    let (width, height) = (1440, 720);

    let mut events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new()
        .with_dimensions(LogicalSize {
            width: width as _,
            height: height as _,
        }).with_title("tanya - hati sample")
        .build(&events_loop)?;

    let engine = render::Engine::new(true);
    let swap_chain = engine.create_swapchain(&window, BUFFER_COUNT);

    let mut fence_values = [0; BUFFER_COUNT as _];

    let (cmd_allocator, _) = engine
        .device
        .create_command_allocator(d3d12::command_list::CmdListType::Direct);
    let (cmd_list, _) = engine.device.create_graphics_command_list(
        d3d12::command_list::CmdListType::Direct,
        cmd_allocator,
        d3d12::PipelineState::null(),
        0,
    );
    cmd_list.close();

    let (frame_fence, _) = engine.device.create_fence(0);
    let frame_event = d3d12::sync::Event::create(false, false);

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

        let frame = swap_chain.begin_frame();
        let (_present_target, present_rtv) = swap_chain.get_present_target(frame);

        cmd_allocator.reset();
        cmd_list.reset(cmd_allocator, d3d12::PipelineState::null());

        cmd_list.clear_render_target_view(present_rtv, [0.4, 0.4, 0.5, 1.0], &[]);

        cmd_list.close();

        engine.queue.execute_command_lists(&[cmd_list.as_list()]);

        swap_chain.end_frame();

        let cur_fence_value = fence_values[frame as usize];
        engine.queue.signal(frame_fence, cur_fence_value);

        if frame_fence.get_value() < cur_fence_value {
            frame_fence.set_event_on_completion(frame_event, cur_fence_value);
            frame_event.wait(1_000_000);
        }

        fence_values[frame as usize] += 1;
    }

    Ok(())
}
