use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex {
    pub fn descriptor() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![
                0 => Float32x3,
                1 => Float32x2
            ];

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>()
                as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

pub fn generate_dome(
    segments: usize,
    radius: f32,
    height: f32,
    arc_degrees: f32,
) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::with_capacity((segments + 1) * 2);
    let mut indices = Vec::with_capacity(segments * 6);

    let arc = arc_degrees.to_radians();

    for segment in 0..=segments {
        let u = segment as f32 / segments as f32;
        let angle = (u - 0.5) * arc;

        let x = radius * angle.sin();
        let z = -radius * angle.cos();

        vertices.push(Vertex {
            position: [x, -height / 2.0, z],
            uv: [u, 1.0],
        });

        vertices.push(Vertex {
            position: [x, height / 2.0, z],
            uv: [u, 0.0],
        });
    }

    for segment in 0..segments {
        let base = (segment * 2) as u32;

        indices.extend_from_slice(&[
            base,
            base + 1,
            base + 2,
            base + 1,
            base + 3,
            base + 2,
        ]);
    }

    (vertices, indices)
}