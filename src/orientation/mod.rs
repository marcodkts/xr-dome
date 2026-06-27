use winit::event::ElementState;
use winit::keyboard::KeyCode;

use crate::app_event::AppEvent;
use crate::integrations::viture::VitureOrientation;

pub mod keyboard;

#[derive(Default, Clone, Copy, Debug)]
pub struct Orientation {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

pub trait OrientationSource {
    fn orientation(&mut self) -> Orientation;
}

pub enum HeadOrientation {
    Keyboard(keyboard::KeyboardOrientation),
    Viture(VitureOrientation),
}

impl HeadOrientation {
    pub fn new(event_proxy: winit::event_loop::EventLoopProxy<AppEvent>) -> Self {
        match VitureOrientation::try_new(event_proxy) {
            Ok(source) => {
                log::info!("VITURE head tracking enabled");
                Self::Viture(source)
            }

            Err(error) => {
                log::warn!("VITURE head tracking unavailable: {error}");
                log::warn!("falling back to keyboard orientation");
                Self::Keyboard(keyboard::KeyboardOrientation::default())
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, state: ElementState) -> bool {
        match self {
            Self::Keyboard(source) => source.handle_key(key, state),

            Self::Viture(source) => source.handle_key(key, state),
        }
    }

    pub fn update(&mut self, delta_seconds: f32) {
        if let Self::Keyboard(source) = self {
            source.update(delta_seconds);
        }
    }

    pub fn reset(&mut self) {
        match self {
            Self::Keyboard(source) => source.reset(),
            Self::Viture(source) => source.reset(),
        }
    }

    pub fn clear_input(&mut self) {
        match self {
            Self::Keyboard(source) => source.clear_input(),
            Self::Viture(source) => source.clear_input(),
        }
    }

    pub fn is_rotating(&self) -> bool {
        match self {
            Self::Keyboard(source) => source.is_rotating(),
            Self::Viture(_) => false,
        }
    }
}

impl OrientationSource for HeadOrientation {
    fn orientation(&mut self) -> Orientation {
        match self {
            Self::Keyboard(source) => source.orientation(),
            Self::Viture(source) => source.orientation(),
        }
    }
}
