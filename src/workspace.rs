use glam::Vec3;
use winit::{dpi::PhysicalPosition, event::ElementState};

use crate::{
    orientation::Orientation,
    renderer::Renderer,
    surface::{SurfaceConfig, SurfaceHit},
};

const SURFACE_PIXEL_WIDTH: u32 = 1915;
const SURFACE_PIXEL_HEIGHT: u32 = 821;

pub struct WorkspaceUpdate {
    pub title: Option<String>,
}

pub struct Workspace {
    main_surface: SurfaceConfig,
    dome_radius: f32,
    cursor_position: Option<PhysicalPosition<f64>>,
    last_hit_cell: Option<(i32, i32)>,
    last_center_hit_cell: Option<(i32, i32)>,
    last_surface_hit: Option<SurfaceHit>,
    left_down_hit: Option<SurfaceHit>,
}

impl Workspace {
    pub fn new(dome_radius: f32, main_surface: SurfaceConfig) -> Self {
        Self {
            main_surface,
            dome_radius,
            cursor_position: None,
            last_hit_cell: None,
            last_center_hit_cell: None,
            last_surface_hit: None,
            left_down_hit: None,
        }
    }

    pub fn surface_mesh(&self, dome_radius: f32) -> crate::surface::SurfaceMesh {
        self.main_surface.build_mesh(dome_radius)
    }

    pub fn set_cursor_position(&mut self, position: Option<PhysicalPosition<f64>>) {
        self.cursor_position = position;

        if position.is_none() {
            self.last_hit_cell = None;
            self.last_surface_hit = None;
            self.left_down_hit = None;
        }
    }

    pub fn clear_title_state(&mut self) {
        self.last_hit_cell = None;
    }

    pub fn handle_mouse_input(&mut self, state: ElementState) {
        match state {
            ElementState::Pressed => {
                self.left_down_hit = self.last_surface_hit;

                if let Some(hit) = self.last_surface_hit {
                    let (x, y) = hit.to_pixel(SURFACE_PIXEL_WIDTH, SURFACE_PIXEL_HEIGHT);

                    log::debug!(
                        "[surface pointer] down u={:.3} v={:.3} pixel=({}, {})",
                        hit.u,
                        hit.v,
                        x,
                        y
                    );
                } else {
                    log::debug!("[surface pointer] down outside surface");
                }
            }

            ElementState::Released => {
                if let Some(hit) = self.last_surface_hit {
                    let (x, y) = hit.to_pixel(SURFACE_PIXEL_WIDTH, SURFACE_PIXEL_HEIGHT);

                    let is_click = self
                        .left_down_hit
                        .map(|down| (down.u - hit.u).abs() < 0.01 && (down.v - hit.v).abs() < 0.01)
                        .unwrap_or(false);

                    if is_click {
                        log::debug!(
                            "[surface pointer] click u={:.3} v={:.3} pixel=({}, {})",
                            hit.u,
                            hit.v,
                            x,
                            y
                        );
                    } else {
                        log::debug!(
                            "[surface pointer] up u={:.3} v={:.3} pixel=({}, {})",
                            hit.u,
                            hit.v,
                            x,
                            y
                        );
                    }
                } else {
                    log::debug!("[surface pointer] up outside surface");
                }

                self.left_down_hit = None;
            }
        }
    }

    pub fn update(
        &mut self,
        renderer: &mut Renderer,
        orientation: Orientation,
        camera_position: Vec3,
    ) -> WorkspaceUpdate {
        let surface_hit = self
            .cursor_position
            .and_then(|cursor_position| {
                renderer.screen_ray(cursor_position, orientation, camera_position)
            })
            .and_then(|ray| self.main_surface.hit_test_ray(self.dome_radius, ray));

        let center_hit = renderer
            .screen_center_ray(orientation, camera_position)
            .and_then(|ray| self.main_surface.hit_test_ray(self.dome_radius, ray));

        self.log_center_hit(center_hit);

        self.last_surface_hit = surface_hit;

        match surface_hit {
            Some(hit) => {
                let cursor_mesh = self
                    .main_surface
                    .build_cursor_mesh(self.dome_radius, hit, 0.35);

                renderer.update_cursor_mesh(&cursor_mesh.vertices, &cursor_mesh.indices);
            }

            None => {
                renderer.clear_cursor_mesh();
            }
        }

        let title = self.update_window_title(surface_hit);

        WorkspaceUpdate { title }
    }

    fn update_window_title(&mut self, surface_hit: Option<SurfaceHit>) -> Option<String> {
        let hit_cell = surface_hit.map(|hit| ((hit.u * 100.0) as i32, (hit.v * 100.0) as i32));

        if hit_cell != self.last_hit_cell {
            self.last_hit_cell = hit_cell;

            return Some(match surface_hit {
                Some(hit) => format!("XR Dome | surface u={:.2} v={:.2}", hit.u, hit.v),
                None => "XR Dome | no surface hit".to_string(),
            });
        }

        None
    }

    fn log_center_hit(&mut self, center_hit: Option<SurfaceHit>) {
        let center_hit_cell =
            center_hit.map(|hit| ((hit.u * 100.0) as i32, (hit.v * 100.0) as i32));

        if center_hit_cell != self.last_center_hit_cell {
            self.last_center_hit_cell = center_hit_cell;

            match center_hit {
                Some(hit) => {
                    log::debug!(
                        "[center ray] hit u={:.3} v={:.3} distance={:.3} position=({:.2}, {:.2}, {:.2})",
                        hit.u,
                        hit.v,
                        hit.distance,
                        hit.position.x,
                        hit.position.y,
                        hit.position.z,
                    );
                }

                None => {
                    log::debug!("[center ray] no hit");
                }
            }
        }
    }
}
