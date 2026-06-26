use crate::dome::Vertex;

#[derive(Clone, Debug)]
pub struct SurfaceConfig {
    pub yaw_center_degrees: f32,
    pub pitch_center_degrees: f32,
    pub yaw_span_degrees: f32,
    pub aspect: f32,
    pub radius_offset: f32,
    pub horizontal_segments: usize,
    pub vertical_segments: usize,
}

pub struct SurfaceMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl SurfaceConfig {
    pub fn main_workspace() -> Self {
        Self {
            yaw_center_degrees: 0.0,
            pitch_center_degrees: 0.0,
            yaw_span_degrees: 120.0,
            aspect: 1915.0 / 821.0,
            radius_offset: 0.03,
            horizontal_segments: 192,
            vertical_segments: 48,
        }
    }

    pub fn build_mesh(&self, dome_radius: f32) -> SurfaceMesh {
        let radius =
            (dome_radius - self.radius_offset).max(0.1);

        let yaw_center =
            self.yaw_center_degrees.to_radians();

        let pitch_center =
            self.pitch_center_degrees.to_radians();

        let yaw_span =
            self.yaw_span_degrees.to_radians();

        /*
         * Mantém a proporção visual do painel.
         *
         * Exemplo:
         * 120° de largura em 21:9 resulta em uma altura
         * angular menor, sem esticar a textura.
         */
        let pitch_span = yaw_span / self.aspect;

        let mut vertices =
            Vec::with_capacity(
                (self.horizontal_segments + 1)
                    * (self.vertical_segments + 1),
            );

        let mut indices =
            Vec::with_capacity(
                self.horizontal_segments
                    * self.vertical_segments
                    * 6,
            );

        for y in 0..=self.vertical_segments {
            let v =
                y as f32 / self.vertical_segments as f32;

            let pitch =
                pitch_center + (0.5 - v) * pitch_span;

            for x in 0..=self.horizontal_segments {
                let u =
                    x as f32 / self.horizontal_segments as f32;

                let yaw =
                    yaw_center + (u - 0.5) * yaw_span;

                /*
                 * Mesma parametrização do domo.
                 * Isso faz a superfície acompanhar a curvatura
                 * da grid esférica.
                 */
                let px =
                    radius * pitch.cos() * yaw.sin();

                let py =
                    radius * pitch.sin();

                let pz =
                    -radius * pitch.cos() * yaw.cos();

                vertices.push(Vertex {
                    position: [px, py, pz],
                    uv: [u, v],
                });
            }
        }

        let row = self.horizontal_segments + 1;

        for y in 0..self.vertical_segments {
            for x in 0..self.horizontal_segments {
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

        SurfaceMesh {
            vertices,
            indices,
        }
    }
}