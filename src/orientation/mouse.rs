use winit::event::{
    DeviceEvent,
    ElementState,
    MouseButton,
    WindowEvent,
};

use super::{Orientation, OrientationSource};

#[derive(Default)]
pub struct MouseOrientation {
    pose: Orientation,
    dragging: bool,
}

impl MouseOrientation {
    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.dragging = *state == ElementState::Pressed;

                println!("dragging: {}", self.dragging);
            }

            _ => {}
        }
    }

    pub fn handle_device_event(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                if self.dragging {
                    self.apply_motion(delta.0, delta.1);
                }
            }

            _ => {}
        }
    }

    pub fn reset(&mut self) {
        self.pose = Orientation::default();
        println!("orientation reset");
    }

    fn apply_motion(&mut self, dx: f64, dy: f64) {
        const SENSITIVITY: f32 = 0.003;

        self.pose.yaw -= dx as f32 * SENSITIVITY;
        self.pose.pitch -= dy as f32 * SENSITIVITY;

        self.clamp_pitch();

        println!(
            "mouse dx={:.2} dy={:.2} yaw={:.2} pitch={:.2}",
            dx,
            dy,
            self.pose.yaw.to_degrees(),
            self.pose.pitch.to_degrees(),
        );
    }

    fn clamp_pitch(&mut self) {
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