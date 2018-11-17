extern crate tanya_render as render;

use self::render::display::Display;
use self::render::vk;

use ash::version::DeviceV1_0;

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
    assert!(display
        .surface()
        .adapter_supported(adapter, main_queue_family));
    let surface_support = display.surface().query_support(adapter);

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

    let fences: [vk::Fence; NUM_FRAMES] = [device.create_fence(true), device.create_fence(true)];
    let frame_ready: [vk::Semaphore; NUM_FRAMES] =
        [device.create_semaphore(), device.create_semaphore()];
    let main_submit_ready: [vk::Semaphore; NUM_FRAMES] =
        [device.create_semaphore(), device.create_semaphore()];
    let main_cmd_pools: [vk::CommandPool; NUM_FRAMES] = [
        device.create_command_pool(main_queue_family),
        device.create_command_pool(main_queue_family),
    ];
    let num_main_buffers = 1;
    let main_cmd_buffers: [Vec<vk::CommandBuffer>; NUM_FRAMES] = [
        device.allocate_command_buffers(main_cmd_pools[0], num_main_buffers),
        device.allocate_command_buffers(main_cmd_pools[1], num_main_buffers),
    ];

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

        if quit {
            break;
        }

        let cur_tick = tick % NUM_FRAMES;

        let frame_img_ready = frame_ready[cur_tick];
        let frame_render_done = main_submit_ready[cur_tick];
        let frame_fence = fences[cur_tick];
        let frame = swapchain.begin_frame(frame_img_ready);
        let frame_image = swapchain.get_image(&frame);

        let main_pool = main_cmd_pools[cur_tick];
        unsafe {
            device.wait_for_fences(&[frame_fence], true, !0).unwrap();
            device.reset_fences(&[frame_fence]).unwrap();
        }
        device.reset_command_pool(main_pool);

        let main_cmd_buffer = main_cmd_buffers[cur_tick][0];

        let begin_info = vk::CommandBufferBeginInfo::default();
        unsafe {
            device
                .begin_command_buffer(main_cmd_buffer, &begin_info)
                .unwrap();

            {
                let present_barrier = vk_sync::ImageBarrier {
                    previous_accesses: vec![vk_sync::AccessType::Present],
                    next_accesses: vec![vk_sync::AccessType::Present],
                    previous_layout: vk_sync::ImageLayout::Optimal,
                    next_layout: vk_sync::ImageLayout::Optimal,
                    discard_contents: true,

                    src_queue_family_index: main_queue_family as _,
                    dst_queue_family_index: main_queue_family as _,
                    image: frame_image,
                    range: vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                };

                let (src_stage, dst_stage, image_barrier) =
                    vk_sync::get_image_memory_barrier(&present_barrier);

                device.cmd_pipeline_barrier(
                    main_cmd_buffer,
                    src_stage,
                    dst_stage,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[image_barrier],
                );
            }

            device.end_command_buffer(main_cmd_buffer).unwrap();

            let main_submit = vk::SubmitInfo {
                wait_semaphore_count: 1,
                p_wait_semaphores: &frame_img_ready as *const _,
                p_wait_dst_stage_mask: &vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                command_buffer_count: 1,
                p_command_buffers: &main_cmd_buffer as *const _,
                signal_semaphore_count: 1,
                p_signal_semaphores: &frame_render_done as *const _,
                ..Default::default()
            };

            device.queue_submit(main_queue, &[main_submit], frame_fence);
        }

        swapchain.end_frame(frame, main_queue, &[frame_render_done]);

        tick += 1;
    }

    unsafe {
        device.device_wait_idle().unwrap();
    }

    for fence in &fences {
        device.destroy_fence(*fence);
    }
    for semaphore in &frame_ready {
        device.destroy_semaphore(*semaphore);
    }
    for semaphore in &main_submit_ready {
        device.destroy_semaphore(*semaphore);
    }
    device.destroy();

    Ok(())
}
