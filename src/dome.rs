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
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

pub fn generate_dome(
    horizontal_segments: usize,
    vertical_segments: usize,
    radius: f32,
    yaw_degrees: f32,
    min_pitch_degrees: f32,
    max_pitch_degrees: f32,
) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices =
        Vec::with_capacity((horizontal_segments + 1) * (vertical_segments + 1));

    let mut indices =
        Vec::with_capacity(horizontal_segments * vertical_segments * 6);

    let yaw_arc = yaw_degrees.to_radians();
    let min_pitch = min_pitch_degrees.to_radians();
    let max_pitch = max_pitch_degrees.to_radians();

    for y in 0..=vertical_segments {
        let v = y as f32 / vertical_segments as f32;

        // v = 0 topo, v = 1 base
        let pitch = max_pitch + (min_pitch - max_pitch) * v;

        for x in 0..=horizontal_segments {
            let u = x as f32 / horizontal_segments as f32;

            let yaw = (u - 0.5) * yaw_arc;

            let px = radius * pitch.cos() * yaw.sin();
            let py = radius * pitch.sin();
            let pz = -radius * pitch.cos() * yaw.cos();

            vertices.push(Vertex {
                position: [px, py, pz],
                uv: [u, v],
            });
        }
    }

    let row = horizontal_segments + 1;

    for y in 0..vertical_segments {
        for x in 0..horizontal_segments {
            let a = (y * row + x) as u32;
            let b = (y * row + x + 1) as u32;
            let c = ((y + 1) * row + x) as u32;
            let d = ((y + 1) * row + x + 1) as u32;

            indices.extend_from_slice(&[
                a, c, b,
                b, c, d,
            ]);
        }
    }

    (vertices, indices)
}