use std::f32::consts::{PI, TAU};

use winit::{event::ElementState, keyboard::KeyCode};

use super::{Orientation, OrientationSource};

const KEYBOARD_ANGULAR_SPEED_DEGREES_PER_SECOND: f32 = 60.0;

pub struct KeyboardOrientation {
    pose: Orientation,

    look_left: bool,
    look_right: bool,
    look_up: bool,
    look_down: bool,

    angular_speed: f32,
}

impl Default for KeyboardOrientation {
    fn default() -> Self {
        Self {
            pose: Orientation::default(),

            look_left: false,
            look_right: false,
            look_up: false,
            look_down: false,

            angular_speed: KEYBOARD_ANGULAR_SPEED_DEGREES_PER_SECOND.to_radians(),
        }
    }
}

impl KeyboardOrientation {
    pub fn handle_key(&mut self, key: KeyCode, state: ElementState) -> bool {
        let pressed = state == ElementState::Pressed;

        match key {
            KeyCode::ArrowLeft => {
                self.look_left = pressed;
            }

            KeyCode::ArrowRight => {
                self.look_right = pressed;
            }

            KeyCode::ArrowUp => {
                self.look_up = pressed;
            }

            KeyCode::ArrowDown => {
                self.look_down = pressed;
            }

            _ => return false,
        }

        true
    }

    pub fn update(&mut self, delta_seconds: f32) {
        let horizontal = self.look_left as i8 - self.look_right as i8;

        let vertical = self.look_up as i8 - self.look_down as i8;

        self.pose.yaw += horizontal as f32 * self.angular_speed * delta_seconds;

        self.pose.pitch += vertical as f32 * self.angular_speed * delta_seconds;

        self.pose.pitch = self
            .pose
            .pitch
            .clamp(-80.0_f32.to_radians(), 80.0_f32.to_radians());

        // Evita que yaw cresça indefinidamente.
        self.pose.yaw = (self.pose.yaw + PI).rem_euclid(TAU) - PI;
    }

    pub fn reset(&mut self) {
        self.pose = Orientation::default();
        self.clear_input();
    }

    pub fn clear_input(&mut self) {
        self.look_left = false;
        self.look_right = false;
        self.look_up = false;
        self.look_down = false;
    }

    pub fn is_rotating(&self) -> bool {
        self.look_left || self.look_right || self.look_up || self.look_down
    }
}

impl OrientationSource for KeyboardOrientation {
    fn orientation(&mut self) -> Orientation {
        self.pose
    }
}
