use wgpu::{util::*, *};

/// A mesh consists of a set of vertices connected by edges in triangles
/// (the indices).
#[derive(Debug)]
pub struct Mesh {
    /// The vertices uploaded to the gpu.
    pub vertex_buffer: Buffer,
    /// The indices uploaded to the gpu. Stored as a list of `u32`s.
    pub index_buffer: Buffer,

    /// The number of vertices present in the buffer.
    pub count: u32,
}

/// Represents a vertex of a triangle, that can easily be uploaded to the rendering device.
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 3],
}

impl Mesh {
    pub fn new(device: &Device, vertices: &[Vertex], indices: &[u32]) -> Self {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Mesh Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            count: vertices.len() as u32,
        }
    }
}

impl Vertex {
    const ATTRIBS: [VertexAttribute; 2] = vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
    ];

    /// Returns the vertex descriptor of how the vertex data is to be interpreted by the shader.
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
