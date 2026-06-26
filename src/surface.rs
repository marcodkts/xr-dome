use std::f32::consts::{PI, TAU};

use glam::Vec3;

use crate::{
    dome::Vertex,
    ray::Ray,
};

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

#[derive(Clone, Copy, Debug)]
pub struct SurfaceHit {
    pub position: Vec3,
    pub u: f32,
    pub v: f32,
    pub distance: f32,
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
        let radius = self.surface_radius(dome_radius);

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
        let pitch_span = self.pitch_span_radians();

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

    pub fn surface_radius(&self, dome_radius: f32) -> f32 {
        (dome_radius - self.radius_offset).max(0.1)
    }

    pub fn pitch_span_radians(&self) -> f32 {
        self.yaw_span_degrees.to_radians() / self.aspect
    }

    pub fn hit_test_ray(
        &self,
        dome_radius: f32,
        ray: Ray,
    ) -> Option<SurfaceHit> {
        let radius = self.surface_radius(dome_radius);

        /*
        * Interseção ray-esfera.
        * A surface está apoiada em uma esfera centrada na origem.
        */

        let a = ray.direction.length_squared();

        if a <= f32::EPSILON {
            return None;
        }

        let b = 2.0 * ray.origin.dot(ray.direction);

        let c =
            ray.origin.length_squared() - radius * radius;

        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrt_discriminant = discriminant.sqrt();

        let t0 =
            (-b - sqrt_discriminant) / (2.0 * a);

        let t1 =
            (-b + sqrt_discriminant) / (2.0 * a);

        let distance = if t0 > 0.001 {
            t0
        } else if t1 > 0.001 {
            t1
        } else {
            return None;
        };

        let position = ray.at(distance);

        /*
        * Converte o ponto 3D da esfera de volta para
        * yaw/pitch usando a mesma parametrização da mesh.
        */

        let normalized = position / radius;

        let pitch = normalized.y
            .clamp(-1.0, 1.0)
            .asin();

        let yaw = normalized.x.atan2(-normalized.z);

        let yaw_center =
            self.yaw_center_degrees.to_radians();

        let pitch_center =
            self.pitch_center_degrees.to_radians();

        let yaw_span =
            self.yaw_span_degrees.to_radians();

        let pitch_span =
            self.pitch_span_radians();

        let yaw_delta =
            angular_delta(yaw, yaw_center);

        let pitch_delta =
            pitch - pitch_center;

        if yaw_delta.abs() > yaw_span * 0.5 {
            return None;
        }

        if pitch_delta.abs() > pitch_span * 0.5 {
            return None;
        }

        /*
        * Mesma relação usada no build_mesh:
        *
        * yaw   = center + (u - 0.5) * yaw_span
        * pitch = center + (0.5 - v) * pitch_span
        */

        let u =
            0.5 + yaw_delta / yaw_span;

        let v =
            0.5 - pitch_delta / pitch_span;

        if !(0.0..=1.0).contains(&u)
            || !(0.0..=1.0).contains(&v)
        {
            return None;
        }

        Some(SurfaceHit {
            position,
            u,
            v,
            distance,
        })
    }
}

fn angular_delta(angle: f32, center: f32) -> f32 {
    (angle - center + PI).rem_euclid(TAU) - PI
}