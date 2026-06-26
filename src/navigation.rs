use glam::{Vec3};
use winit::{
    event::ElementState,
    keyboard::KeyCode,
};

use crate::orientation::Orientation;

pub struct Navigation {
    position: Vec3,
    initial_position: Vec3,

    dome_radius: f32,
    safety_margin: f32,

    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    sprint: bool,

    speed: f32,
}

impl Navigation {
    pub fn new(position: Vec3, dome_radius: f32) -> Self {
        Self {
            position,
            initial_position: position,

            dome_radius,
            safety_margin: 0.2,

            forward: false,
            backward: false,
            left: false,
            right: false,
            up: false,
            down: false,
            sprint: false,

            speed: 1.5,
        }
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn reset(&mut self) {
        self.position = self.initial_position;
        self.clear_input();
    }

    pub fn clear_input(&mut self) {
        self.forward = false;
        self.backward = false;
        self.left = false;
        self.right = false;
        self.up = false;
        self.down = false;
        self.sprint = false;
    }

    pub fn is_moving(&self) -> bool {
        self.forward
            || self.backward
            || self.left
            || self.right
            || self.up
            || self.down
    }

    pub fn handle_key(
        &mut self,
        key: KeyCode,
        state: ElementState,
    ) -> bool {
        let pressed = state == ElementState::Pressed;

        match key {
            KeyCode::KeyW => {
                self.forward = pressed;
            }

            KeyCode::KeyS => {
                self.backward = pressed;
            }

            KeyCode::KeyA => {
                self.left = pressed;
            }

            KeyCode::KeyD => {
                self.right = pressed;
            }

            KeyCode::KeyE => {
                self.up = pressed;
            }

            KeyCode::KeyQ => {
                self.down = pressed;
            }

            KeyCode::ShiftLeft
            | KeyCode::ShiftRight => {
                self.sprint = pressed;
            }

            _ => return false,
        }

        true
    }

    pub fn update(
        &mut self,
        delta_seconds: f32,
        orientation: Orientation,
    ) {
        let (sin_yaw, cos_yaw) =
            orientation.yaw.sin_cos();

        /*
         * Movimento acompanha somente o yaw.
         * Olhar para cima não faz o observador voar.
         */

        let forward = Vec3::new(
            -sin_yaw,
            0.0,
            -cos_yaw,
        );

        let right = Vec3::new(
            cos_yaw,
            0.0,
            -sin_yaw,
        );

        let mut direction = Vec3::ZERO;

        if self.forward {
            direction += forward;
        }

        if self.backward {
            direction -= forward;
        }

        if self.right {
            direction += right;
        }

        if self.left {
            direction -= right;
        }

        if self.up {
            direction += Vec3::Y;
        }

        if self.down {
            direction -= Vec3::Y;
        }

        if direction.length_squared() > 0.0 {
            direction = direction.normalize();

            let speed = if self.sprint {
                self.speed * 2.5
            } else {
                self.speed
            };

            self.position +=
                direction * speed * delta_seconds;
        }

        self.apply_bounds();
    }

    fn apply_bounds(&mut self) {
        let maximum_distance =
            (self.dome_radius - self.safety_margin).max(0.1);

        let distance = self.position.length();

        if distance > maximum_distance {
            self.position =
                self.position.normalize() * maximum_distance;
        }
    }

    pub fn set_dome_radius(&mut self, radius: f32) {
        self.dome_radius =
            radius.max(self.safety_margin + 0.1);

        // Reposiciona imediatamente caso a redução
        // do domo deixe a câmera do lado de fora.
        self.apply_bounds();
    }
}