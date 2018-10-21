extern crate glutin;
extern crate grr;
extern crate tanya_ui as ui;

use self::ui::{NodeId, Ui};
use glutin::GlContext;

mod render;

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("Hello, world!")
        .with_dimensions((1024, 768).into());
    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_srgb(true);

    let window = glutin::GlWindow::new(window, context, &events_loop).unwrap();
    let glutin::dpi::LogicalSize { width, height } = window.get_inner_size().unwrap();

    unsafe {
        window.make_current().unwrap();
    }

    let grr = grr::Device::new(|symbol| window.get_proc_address(symbol) as *const _);
    let (pipeline, vertex_array) = render::build_layout_debug_pipeline(&grr);

    let mut ui = ui::Ui::new();
    let app = ui::App {
        button0: ui.gen_key(),
        button1: ui.gen_key(),
    };

    ui.build(app);
    ui.layout(width as _, height as _);

    println!("{:#?}", ui.tree);
    println!("{:#?}", ui.instance_lut);

    for node in ui.tree.iter() {
        let id = node.data;
        let instance = &ui.instances[&id];
        let layout = &instance.layout;
        println!("{:?}", (id, layout.borrow_mut().get_layout()));
        println!("{:#?}", (id, instance.geometry));
    }

    fn build_debug_render_tree(
        ui: &Ui,
        width: f32,
        height: f32,
        idx: NodeId,
        color_gen: &mut random_color::RandomColor,
        rects: &mut Vec<f32>,
    ) {
        {
            let id = ui.tree[idx].data;
            let instance = ui.instances.get(&id).unwrap();
            let geometry = instance.geometry;
            let color = color_gen.to_rgb_array();
            let r = color[0] as f32 / 255.0;
            let g = color[1] as f32 / 255.0;
            let b = color[2] as f32 / 255.0;
            let left = (2.0 * geometry.left - width) / width;
            let right = (2.0 * geometry.right - width) / width;
            let top = -(2.0 * geometry.top - height) / height;
            let bottom = -(2.0 * geometry.bottom - height) / height;

            rects.extend(&[left, top, r, g, b]);
            rects.extend(&[right, top, r, g, b]);
            rects.extend(&[left, bottom, r, g, b]);
            rects.extend(&[left, bottom, r, g, b]);
            rects.extend(&[right, top, r, g, b]);
            rects.extend(&[right, bottom, r, g, b]);
        };
        for child in idx.children(&ui.tree) {
            build_debug_render_tree(ui, width, height, child, color_gen, rects);
        }
    }

    let mut debug_rects = Vec::new();
    let mut color_gen = random_color::RandomColor::new();
    build_debug_render_tree(
        &ui,
        width as _,
        height as _,
        ui.instance_lut[&ui.root],
        &mut color_gen,
        &mut debug_rects,
    );

    let rect_data = {
        let len = (std::mem::size_of::<f32>() * debug_rects.len()) as u64;

        let buffer = grr.create_buffer(
            len,
            grr::MemoryFlags::CPU_MAP_WRITE | grr::MemoryFlags::COHERENT,
        );

        let data = grr.map_buffer::<f32>(&buffer, 0..len, grr::MappingFlags::empty());
        data.clone_from_slice(&debug_rects);
        grr.unmap_buffer(&buffer);

        buffer
    };

    let mut running = true;
    while running {
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => running = false,
                _ => (),
            },
            _ => (),
        });

        grr.bind_pipeline(&pipeline);
        grr.bind_vertex_array(&vertex_array);

        grr.bind_vertex_buffers(
            &vertex_array,
            0,
            &[grr::VertexBufferView {
                buffer: &rect_data,
                offset: 0,
                stride: (std::mem::size_of::<f32>() * 5) as _,
            }],
        );

        grr.set_viewport(
            0,
            &[grr::Viewport {
                x: 0.0,
                y: 0.0,
                w: width as _,
                h: height as _,
                n: 0.0,
                f: 1.0,
            }],
        );
        grr.set_scissor(
            0,
            &[grr::Region {
                x: 0,
                y: 0,
                w: width as _,
                h: height as _,
            }],
        );

        grr.clear_attachment(
            grr::Framebuffer::DEFAULT,
            grr::ClearAttachment::ColorFloat(0, [0.5, 0.5, 0.5, 1.0]),
        );

        grr.draw(
            grr::Primitive::Triangles,
            0..(debug_rects.len() / 5) as u32,
            0..1,
        );

        window.swap_buffers().unwrap();
    }
}
