use d3d12::dxgi;
use failure::Error;
use tanya::render;
use winapi::shared::{dxgiformat, dxgitype, winerror};
use winit::os::windows::WindowExt;
use winit::{dpi::LogicalSize, WindowEvent};

fn select_adapter(factory: dxgi::Factory4) -> dxgi::Adapter1 {
    let mut id = 0;
    loop {
        let (adapter, hr) = factory.enumerate_adapters(id);
        if hr == winerror::DXGI_ERROR_NOT_FOUND {
            panic!("unable to find adapter")
        }
        id += 1;

        // TODO: check support

        return adapter;
    }
}

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

    let mut engine = render::Engine::new(true);

    let adapter = select_adapter(engine.factory);
    let (device, _) = d3d12::Device::create(adapter, d3d12::FeatureLevel::L12_0);

    let (queue, _) = device.create_command_queue(
        d3d12::command_list::CmdListType::Direct,
        d3d12::queue::Priority::Normal,
        d3d12::queue::CommandQueueFlags::empty(),
        0,
    );

    let swapchain = {
        let desc = dxgi::SwapchainDesc {
            width,
            height,
            format: dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
            stereo: false,
            sample: d3d12::SampleDesc {
                count: 1,
                quality: 0,
            },
            buffer_usage: dxgitype::DXGI_USAGE_RENDER_TARGET_OUTPUT,
            buffer_count: BUFFER_COUNT as _,
            scaling: dxgi::Scaling::Stretch,
            swap_effect: dxgi::SwapEffect::FlipDiscard,
            alpha_mode: dxgi::AlphaMode::Ignore,
            flags: 0,
        };

        let (swapchain, _) = engine.factory.as2().create_swapchain_for_hwnd(
            queue,
            window.get_hwnd() as *mut _,
            &desc,
        );
        let (swapchain3, _): d3d12::D3DResult<dxgi::SwapChain3> = unsafe { swapchain.cast() };
        unsafe {
            swapchain.destroy();
        }

        swapchain3
    };

    let (backbuffer_rtv_heap, _) = device.create_descriptor_heap(
        BUFFER_COUNT,
        d3d12::descriptor::HeapType::Rtv,
        d3d12::descriptor::HeapFlags::empty(),
        0,
    );

    let mut fence_values = [0; BUFFER_COUNT as _];

    let backbuffers = {
        let initial = backbuffer_rtv_heap.start_cpu_descriptor();
        let desc = d3d12::descriptor::RenderTargetViewDesc::texture_2d(
            dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
            0,
            0,
        );
        let handle_size = device.get_descriptor_increment_size(d3d12::descriptor::HeapType::Rtv);

        (0..BUFFER_COUNT)
            .map(|i| {
                let rtv = d3d12::CpuDescriptor {
                    ptr: initial.ptr + (i * handle_size) as usize,
                };
                let (resource, _) = swapchain.as0().get_buffer(i);
                device.create_render_target_view(resource, &desc, rtv);
                (resource, rtv)
            }).collect::<Vec<_>>()
    };

    let (cmd_allocator, _) =
        device.create_command_allocator(d3d12::command_list::CmdListType::Direct);
    let (cmd_list, _) = device.create_graphics_command_list(
        d3d12::command_list::CmdListType::Direct,
        cmd_allocator,
        d3d12::PipelineState::null(),
        0,
    );
    cmd_list.close();

    let (frame_fence, _) = device.create_fence(0);
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

        let frame = swapchain.get_current_back_buffer_index();
        let (present_target, present_rtv) = backbuffers[frame as usize];

        cmd_allocator.reset();
        cmd_list.reset(cmd_allocator, d3d12::PipelineState::null());

        cmd_list.clear_render_target_view(present_rtv, [0.4, 0.4, 0.5, 1.0], &[]);

        cmd_list.close();

        unsafe {
            queue.ExecuteCommandLists(1, &mut cmd_list.as_mut_ptr() as *mut *mut _ as *mut *mut _);
        }

        swapchain.as0().present(0, 0);

        let cur_fence_value = fence_values[frame as usize];
        queue.signal(frame_fence, cur_fence_value);

        if frame_fence.get_value() < cur_fence_value {
            frame_fence.set_event_on_completion(frame_event, cur_fence_value);
            frame_event.wait(1_000_000);
        }

        fence_values[frame as usize] += 1;
    }

    /*
    unsafe {
        for buffer in backbuffers {
            buffer.destroy();
        }
        queue.destroy();
        swapchain.destroy();
        device.destroy();
        adapter.destroy();
    }
    */

    Ok(())
}
