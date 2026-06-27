use serde::{Deserialize, Serialize};

use crate::dome::Vertex;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomeConfig {
    pub horizontal_segments: u32,
    pub vertical_segments: u32,
    pub radius: f32,
    pub yaw_degrees: f32,
    pub min_pitch_degrees: f32,
    pub max_pitch_degrees: f32,
}

impl Default for DomeConfig {
    fn default() -> Self {
        Self {
            horizontal_segments: 512,
            vertical_segments: 128,
            radius: 2.5,
            yaw_degrees: 140.0,
            min_pitch_degrees: -30.0,
            max_pitch_degrees: 30.0,
        }
    }
}

impl DomeConfig {
    pub fn build_mesh(&self) -> (Vec<Vertex>, Vec<u32>) {
        crate::dome::generate_dome(
            self.horizontal_segments as usize,
            self.vertical_segments as usize,
            self.radius,
            self.yaw_degrees,
            self.min_pitch_degrees,
            self.max_pitch_degrees,
        )
    }
}
