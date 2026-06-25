use winit::event::{
    ElementState,
    MouseButton,
    WindowEvent,
};

use super::{Orientation, OrientationSource};

#[derive(Default)]
pub struct MouseOrientation {
    pose: Orientation,
    dragging: bool,
    last_position: Option<(f64, f64)>,
}

impl MouseOrientation {
    pub fn handle_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.dragging = *state == ElementState::Pressed;

                if !self.dragging {
                    self.last_position = None;
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                if !self.dragging {
                    self.last_position = Some((position.x, position.y));
                    return;
                }

                if let Some((last_x, last_y)) = self.last_position {
                    let dx = position.x - last_x;
                    let dy = position.y - last_y;

                    self.apply_motion(dx, dy);
                }

                self.last_position = Some((position.x, position.y));
            }

            _ => {}
        }
    }

    fn apply_motion(&mut self, dx: f64, dy: f64) {
        const SENSITIVITY: f32 = 0.004;

        self.pose.yaw -= dx as f32 * SENSITIVITY;
        self.pose.pitch -= dy as f32 * SENSITIVITY;

        self.pose.pitch = self.pose.pitch.clamp(
            -80.0_f32.to_radians(),
            80.0_f32.to_radians(),
        );
    }
}

impl OrientationSource for MouseOrientation {
    fn orientation(&mut self) -> Orientation {
        self.pose
    }
}