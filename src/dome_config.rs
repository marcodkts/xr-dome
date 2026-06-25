use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

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
            radius: 3.2,
            yaw_degrees: 360.0,
            min_pitch_degrees: -75.0,
            max_pitch_degrees: 75.0,
        }
    }
}

impl DomeConfig {
    pub fn clamp(&mut self) {
        self.horizontal_segments = self.horizontal_segments.clamp(32, 1024);
        self.vertical_segments = self.vertical_segments.clamp(8, 256);

        self.radius = self.radius.clamp(1.0, 10.0);
        self.yaw_degrees = self.yaw_degrees.clamp(30.0, 360.0);

        self.min_pitch_degrees = self.min_pitch_degrees.clamp(-89.0, 0.0);
        self.max_pitch_degrees = self.max_pitch_degrees.clamp(0.0, 89.0);

        if self.min_pitch_degrees >= self.max_pitch_degrees {
            self.min_pitch_degrees = self.max_pitch_degrees - 1.0;
        }
    }

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

#[derive(Clone)]
pub struct SharedDomeConfig {
    config: Arc<Mutex<DomeConfig>>,
    dirty: Arc<AtomicBool>,
}

impl SharedDomeConfig {
    pub fn new(config: DomeConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            dirty: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn get(&self) -> DomeConfig {
        self.config
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn set(&self, mut config: DomeConfig) {
        config.clamp();

        *self
            .config
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = config;

        self.dirty.store(true, Ordering::SeqCst);
    }

    pub fn take_dirty(&self) -> bool {
        self.dirty.swap(false, Ordering::SeqCst)
    }
}