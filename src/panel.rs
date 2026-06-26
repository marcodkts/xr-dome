use crate::dome::Vertex;

pub fn generate_spherical_panel(
    yaw_degrees: f32,
    aspect: f32,
    dome_radius: f32,
    surface_offset: f32,
    horizontal_segments: usize,
    vertical_segments: usize,
) -> (Vec<Vertex>, Vec<u32>) {
    let radius = (dome_radius - surface_offset).max(0.1);

    let yaw_span = yaw_degrees.to_radians();

    /*
     * Mantém a proporção visual do painel.
     *
     * Largura angular = yaw_span.
     * Altura angular = largura angular / aspect.
     */
    let pitch_span = yaw_span / aspect;

    let mut vertices =
        Vec::with_capacity((horizontal_segments + 1) * (vertical_segments + 1));

    let mut indices =
        Vec::with_capacity(horizontal_segments * vertical_segments * 6);

    for y in 0..=vertical_segments {
        let v = y as f32 / vertical_segments as f32;

        // v = 0 topo, v = 1 base
        let pitch = (0.5 - v) * pitch_span;

        for x in 0..=horizontal_segments {
            let u = x as f32 / horizontal_segments as f32;
            let yaw = (u - 0.5) * yaw_span;

            /*
             * Mesma parametrização do domo.
             * Isso faz o painel acompanhar a curvatura da grid.
             */
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