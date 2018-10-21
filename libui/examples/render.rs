use grr;
use std::mem;

const VERTEX_SRC: &str = r#"
    #version 450 core
    layout (location = 0) in vec2 v_pos;
    layout (location = 1) in vec3 v_color;
    layout (location = 0) out vec3 a_color;
    void main() {
        a_color = v_color;
        gl_Position = vec4(v_pos, 0.0, 1.0);
    }
"#;

const FRAGMENT_SRC: &str = r#"
    #version 450 core
    layout (location = 0) in vec3 a_color;
    out vec4 f_color;
    void main() {
       f_color = vec4(a_color, 1.0);
    }
"#;

pub fn build_layout_debug_pipeline(device: &grr::Device) -> (grr::Pipeline, grr::VertexArray) {
    let vs = device.create_shader(grr::ShaderStage::Vertex, VERTEX_SRC.as_bytes());
    let fs = device.create_shader(grr::ShaderStage::Fragment, FRAGMENT_SRC.as_bytes());

    let pipeline = device.create_graphics_pipeline(grr::GraphicsPipelineDesc {
        vertex_shader: &vs,
        tessellation_control_shader: None,
        tessellation_evaluation_shader: None,
        geometry_shader: None,
        fragment_shader: Some(&fs),
    });

    let vertex_array = device.create_vertex_array(&[
        grr::VertexAttributeDesc {
            location: 0,
            binding: 0,
            format: grr::VertexFormat::Xy32Float,
            offset: 0,
            input_rate: grr::InputRate::Vertex,
        },
        grr::VertexAttributeDesc {
            location: 1,
            binding: 0,
            format: grr::VertexFormat::Xyz32Float,
            offset: (2 * mem::size_of::<f32>()) as _,
            input_rate: grr::InputRate::Vertex,
        },
    ]);

    (pipeline, vertex_array)
}
