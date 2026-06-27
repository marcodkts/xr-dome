use std::{
    mem,
    sync::{Arc, Mutex, OnceLock, Weak},
};

use libloading::Library;
use winit::event::ElementState;
use winit::event_loop::EventLoopProxy;
use winit::keyboard::KeyCode;

use crate::app_event::AppEvent;
use crate::orientation::{Orientation, OrientationSource};

type CallbackImu = extern "C" fn(*mut u8, u16, u32);
type CallbackMcu = extern "C" fn(u16, *mut u8, u16, u32);

type InitFn = unsafe extern "C" fn(CallbackImu, CallbackMcu) -> bool;
type DeinitFn = unsafe extern "C" fn();
type SetImuFn = unsafe extern "C" fn(bool) -> i32;

static EVENT_PROXY: OnceLock<Mutex<Option<EventLoopProxy<AppEvent>>>> = OnceLock::new();
static ACTIVE_STATE: OnceLock<Mutex<Option<Weak<Mutex<VitureState>>>>> = OnceLock::new();
static LOG_IMU: OnceLock<bool> = OnceLock::new();

#[derive(Default)]
struct VitureState {
    latest: Orientation,
    offset: Orientation,
}

pub struct VitureSdk {
    _library: Library,
    init: InitFn,
    deinit: DeinitFn,
    set_imu: SetImuFn,
}

pub struct VitureOrientation {
    sdk: VitureSdk,
    state: Arc<Mutex<VitureState>>,
}

impl VitureSdk {
    fn load() -> Result<Self, String> {
        let library = open_library()?;

        // SAFETY: symbol names come from the vendor header and are loaded
        // for the lifetime of the library handle stored in the struct.
        let init = unsafe {
            *library
                .get::<InitFn>(b"init\0")
                .map_err(|error| error.to_string())?
        };

        // SAFETY: same as above.
        let deinit = unsafe {
            *library
                .get::<DeinitFn>(b"deinit\0")
                .map_err(|error| error.to_string())?
        };

        // SAFETY: same as above.
        let set_imu = unsafe {
            *library
                .get::<SetImuFn>(b"set_imu\0")
                .map_err(|error| error.to_string())?
        };

        Ok(Self {
            _library: library,
            init,
            deinit,
            set_imu,
        })
    }
}

impl VitureOrientation {
    pub fn try_new(event_proxy: EventLoopProxy<AppEvent>) -> Result<Self, String> {
        let sdk = VitureSdk::load()?;
        let state = Arc::new(Mutex::new(VitureState::default()));

        let _ = LOG_IMU.get_or_init(|| std::env::var_os("VITURE_LOG_IMU").is_some());
        set_event_proxy(Some(event_proxy));
        set_active_state(Some(Arc::downgrade(&state)));

        let ok = unsafe { (sdk.init)(imu_callback, mcu_callback) };

        if !ok {
            unsafe {
                (sdk.deinit)();
            }
            set_active_state(None);
            set_event_proxy(None);
            return Err("VITURE init returned false".to_string());
        }

        let result = unsafe { (sdk.set_imu)(true) };

        if result < 0 {
            unsafe {
                (sdk.deinit)();
            }
            set_active_state(None);
            set_event_proxy(None);
            return Err(format!("set_imu(true) failed with code {result}"));
        }

        Ok(Self { sdk, state })
    }

    pub fn handle_key(&mut self, _key: KeyCode, _state: ElementState) -> bool {
        false
    }

    pub fn reset(&mut self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        state.offset = Orientation {
            yaw: -state.latest.yaw,
            pitch: -state.latest.pitch,
            roll: -state.latest.roll,
        };
    }

    pub fn clear_input(&mut self) {}

    fn corrected_orientation(&self) -> Orientation {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        normalize_orientation(Orientation {
            yaw: state.latest.yaw + state.offset.yaw,
            pitch: state.latest.pitch + state.offset.pitch,
            roll: state.latest.roll + state.offset.roll,
        })
    }
}

impl Drop for VitureOrientation {
    fn drop(&mut self) {
        set_active_state(None);
        set_event_proxy(None);

        unsafe {
            (self.sdk.set_imu)(false);
            (self.sdk.deinit)();
        }
    }
}

impl OrientationSource for VitureOrientation {
    fn orientation(&mut self) -> Orientation {
        self.corrected_orientation()
    }
}

extern "C" fn imu_callback(data: *mut u8, len: u16, _ts: u32) {
    let Some(state) = active_state() else {
        return;
    };

    let Some(bytes) = (unsafe {
        (!data.is_null()).then(|| std::slice::from_raw_parts(data as *const u8, len as usize))
    }) else {
        return;
    };

    let Some(latest) = decode_orientation(bytes) else {
        return;
    };

    {
        let mut state = state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.latest = latest;
    }
    if let Some(proxy) = event_proxy() {
        let _ = proxy.send_event(AppEvent::VitureImuUpdated);
    }
}

extern "C" fn mcu_callback(_msgid: u16, _data: *mut u8, _len: u16, _ts: u32) {}

fn decode_orientation(bytes: &[u8]) -> Option<Orientation> {
    if bytes.len() < 12 {
        return None;
    }

    let roll = read_f32_be(bytes, 0)?;
    let pitch = read_f32_be(bytes, 4)?;
    let yaw = read_f32_be(bytes, 8)?;

    if log_imu_enabled() {
        log::debug!("[viture imu] roll={roll:.3} pitch={pitch:.3} yaw={yaw:.3}");
    }

    let convert = if looks_like_degrees(roll, pitch, yaw) {
        std::f32::consts::PI / 180.0
    } else {
        1.0
    };

    Some(normalize_orientation(Orientation {
        // O sample do SDK entrega roll/pitch/yaw nessa ordem.
        // Mantemos yaw direto para o giro horizontal.
        yaw: yaw * convert,
        // O eixo vertical da VITURE vem invertido em relação à câmera.
        pitch: -pitch * convert,
        // O tilt lateral da cabeça continua invertido para ficar natural na câmera.
        roll: -roll * convert,
    }))
}

fn read_f32_be(bytes: &[u8], start: usize) -> Option<f32> {
    let slice = bytes.get(start..start + mem::size_of::<f32>())?;
    let array: [u8; 4] = slice.try_into().ok()?;
    Some(f32::from_be_bytes(array))
}

fn normalize_orientation(mut orientation: Orientation) -> Orientation {
    orientation.yaw = normalize_angle(orientation.yaw);
    orientation.roll = normalize_angle(orientation.roll);
    orientation.pitch = orientation
        .pitch
        .clamp(-80.0_f32.to_radians(), 80.0_f32.to_radians());
    orientation
}

fn normalize_angle(angle: f32) -> f32 {
    use std::f32::consts::{PI, TAU};

    (angle + PI).rem_euclid(TAU) - PI
}

fn looks_like_degrees(roll: f32, pitch: f32, yaw: f32) -> bool {
    [roll, pitch, yaw]
        .iter()
        .any(|value| value.abs() > std::f32::consts::TAU * 1.5)
}

fn log_imu_enabled() -> bool {
    *LOG_IMU.get_or_init(|| std::env::var_os("VITURE_LOG_IMU").is_some())
}

fn open_library() -> Result<Library, String> {
    let mut candidates = Vec::new();

    if let Ok(path) = std::env::var("VITURE_SDK_LIB") {
        candidates.push(path);
    }

    if let Ok(root) = std::env::var("VITURE_SDK_ROOT") {
        candidates.push(format!("{root}/libs/libviture_one_sdk.so"));
    }

    candidates.push("./viture_sdk/libs/libviture_one_sdk.so".to_string());
    candidates.push("./viture_sdk/libviture_one_sdk.so".to_string());
    candidates.push("libviture_one_sdk.so".to_string());

    let mut errors = Vec::new();

    for candidate in candidates {
        // SAFETY: library loading delegates to the OS loader.
        match unsafe { Library::new(&candidate) } {
            Ok(library) => {
                log::info!("loaded VITURE SDK: {candidate}");
                return Ok(library);
            }

            Err(error) => {
                errors.push(format!("{candidate}: {error}"));
            }
        }
    }

    Err(format!(
        "could not load libviture_one_sdk.so\n{}",
        errors.join("\n")
    ))
}

fn set_event_proxy(proxy: Option<EventLoopProxy<AppEvent>>) {
    let slot = EVENT_PROXY.get_or_init(|| Mutex::new(None));
    *slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = proxy;
}

fn event_proxy() -> Option<EventLoopProxy<AppEvent>> {
    EVENT_PROXY
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

fn set_active_state(state: Option<Weak<Mutex<VitureState>>>) {
    let slot = ACTIVE_STATE.get_or_init(|| Mutex::new(None));
    *slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = state;
}

fn active_state() -> Option<Arc<Mutex<VitureState>>> {
    ACTIVE_STATE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .as_ref()
        .and_then(Weak::upgrade)
}
