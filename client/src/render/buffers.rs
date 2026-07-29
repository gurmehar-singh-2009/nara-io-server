use bytemuck::{Pod, Zeroable};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}

// make gamestate return entity instances and just pass that to renderer
// also make sure to push stuff earlier so it renders underneath
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct EntityInstance {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub rotation: f32,

    /// 0 = Circle
    /// 1 = Box
    /// 2 = Trapezoid
    /// 3 = Polygon
    pub shape_type: u32,

    /// 3 = Triangle
    /// 4 = Square
    /// 5 = Pentagon
    pub sides: u32,
    pub fill_color: [f32; 4],
    pub border_color: [f32; 4],
    pub border_thickness: f32,
    pub extra_param: f32,
}

impl EntityInstance {
    pub fn desc() -> VertexBufferLayout<'static> {
        const ATTRIBS: [VertexAttribute; 9] = wgpu::vertex_attr_array![
            0 => Float32x2, // pos
            1 => Float32x2, // size
            2 => Float32,   // rotation
            3 => Uint32,    // shape_type
            4 => Uint32,    // sides
            5 => Float32x4, // fill_color
            6 => Float32x4, // border_color
            7 => Float32,   // border_thickness
            8 => Float32,   // extra_param
        ];

        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 2],
    pub zoom: f32,
    pub aspect_ratio: f32,
}
