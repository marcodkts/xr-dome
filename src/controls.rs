use glam::Vec3;
use winit::{event::ElementState, event_loop::EventLoopProxy, keyboard::KeyCode};

use crate::{
    app_event::AppEvent,
    navigation::Navigation,
    orientation::{HeadOrientation, Orientation, OrientationSource},
};

pub struct DesktopControls {
    navigation: Navigation,
    head_orientation: HeadOrientation,
}

impl DesktopControls {
    pub fn new(event_proxy: EventLoopProxy<AppEvent>, dome_radius: f32) -> Self {
        Self {
            navigation: Navigation::new(Vec3::ZERO, dome_radius),
            head_orientation: HeadOrientation::new(event_proxy),
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, state: ElementState) -> bool {
        self.navigation.handle_key(key, state) || self.head_orientation.handle_key(key, state)
    }

    pub fn update_head(&mut self, delta_seconds: f32) {
        self.head_orientation.update(delta_seconds);
    }

    pub fn update_navigation(&mut self, delta_seconds: f32, orientation: Orientation) {
        self.navigation.update(delta_seconds, orientation);
    }

    pub fn orientation(&mut self) -> Orientation {
        self.head_orientation.orientation()
    }

    pub fn position(&self) -> Vec3 {
        self.navigation.position()
    }

    pub fn clear_input(&mut self) {
        self.navigation.clear_input();
        self.head_orientation.clear_input();
    }

    pub fn reset(&mut self) {
        self.navigation.reset();
        self.head_orientation.reset();
    }

    pub fn reset_orientation(&mut self) {
        self.head_orientation.reset();
    }

    pub fn is_active(&self) -> bool {
        self.navigation.is_moving() || self.head_orientation.is_rotating()
    }
}
