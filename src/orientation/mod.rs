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