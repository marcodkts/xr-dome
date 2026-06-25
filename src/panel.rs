use crate::dome::Vertex;

pub fn generate_curved_panel(
    width: f32,
    height: f32,
    radius: f32,
    horizontal_segments: usize,
    vertical_segments: usize,
) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices =
        Vec::with_capacity((horizontal_segments + 1) * (vertical_segments + 1));

    let mut indices =
        Vec::with_capacity(horizontal_segments * vertical_segments * 6);

    let yaw_span = width / radius;
    let half_height = height / 2.0;

    for y in 0..=vertical_segments {
        let v = y as f32 / vertical_segments as f32;
        let py = half_height - v * height;

        for x in 0..=horizontal_segments {
            let u = x as f32 / horizontal_segments as f32;
            let yaw = (u - 0.5) * yaw_span;

            let px = radius * yaw.sin();
            let pz = -radius * yaw.cos();

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