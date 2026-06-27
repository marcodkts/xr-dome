use crate::{dome_config::DomeConfig, surface::SurfaceConfig};

pub struct SceneConfig {
    pub dome: DomeConfig,
    pub workspace: SurfaceConfig,
}

impl SceneConfig {
    pub fn from_env() -> Self {
        let dome = DomeConfig {
            horizontal_segments: env_u32("XR_DOME_DOME_HORIZONTAL_SEGMENTS", 512),
            vertical_segments: env_u32("XR_DOME_DOME_VERTICAL_SEGMENTS", 128),
            radius: env_f32("XR_DOME_OBSERVER_DISTANCE_M", 2.5),
            yaw_degrees: env_f32("XR_DOME_DOME_YAW_DEGREES", 140.0),
            min_pitch_degrees: env_f32("XR_DOME_DOME_MIN_PITCH_DEGREES", -30.0),
            max_pitch_degrees: env_f32("XR_DOME_DOME_MAX_PITCH_DEGREES", 30.0),
        };

        let workspace = SurfaceConfig {
            yaw_center_degrees: env_f32("XR_DOME_WORKSPACE_YAW_CENTER_DEGREES", 0.0),
            pitch_center_degrees: env_f32("XR_DOME_WORKSPACE_PITCH_CENTER_DEGREES", 0.0),
            yaw_span_degrees: env_f32("XR_DOME_WORKSPACE_YAW_DEGREES", 140.0),
            pitch_span_degrees: env_f32("XR_DOME_WORKSPACE_PITCH_DEGREES", 60.0),
            radius_offset: env_f32("XR_DOME_WORKSPACE_RADIUS_OFFSET", 0.03),
            horizontal_segments: env_usize("XR_DOME_WORKSPACE_HORIZONTAL_SEGMENTS", 192),
            vertical_segments: env_usize("XR_DOME_WORKSPACE_VERTICAL_SEGMENTS", 48),
        };

        Self { dome, workspace }
    }
}

fn env_f32(name: &str, default: f32) -> f32 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(default)
}

fn env_u32(name: &str, default: u32) -> u32 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(default)
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}
