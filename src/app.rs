use std::{sync::Arc, time::Instant};

use winit::{
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoopProxy, EventLoopWindowTarget},
    keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
    window::{Fullscreen, Window, WindowId},
};

use crate::{
    app_event::AppEvent, controls::DesktopControls, renderer::Renderer, settings::SceneConfig,
    workspace::Workspace,
};

pub const WINDOW_TITLE: &str = "XR Dome";

pub struct App {
    window: Arc<Window>,
    renderer: Renderer,
    controls: DesktopControls,
    workspace: Workspace,
    last_frame: Instant,
}

impl App {
    pub fn new(window: Arc<Window>, event_proxy: EventLoopProxy<AppEvent>) -> Self {
        let scene = SceneConfig::from_env();
        let dome_config = scene.dome.clone();
        let (vertices, indices) = dome_config.build_mesh();

        let workspace = Workspace::new(dome_config.radius, scene.workspace);

        let surface_mesh = workspace.surface_mesh(dome_config.radius);

        let renderer = pollster::block_on(Renderer::new(
            Arc::clone(&window),
            &vertices,
            &indices,
            &surface_mesh.vertices,
            &surface_mesh.indices,
            &dome_config,
            &scene.visual,
        ));

        let controls = DesktopControls::new(event_proxy, dome_config.radius);

        Self {
            window,
            renderer,
            controls,
            workspace,
            last_frame: Instant::now(),
        }
    }

    pub fn window_id(&self) -> WindowId {
        self.window.id()
    }

    pub fn handle_window_event(
        &mut self,
        event: WindowEvent,
        event_loop: &EventLoopWindowTarget<AppEvent>,
    ) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.workspace.set_cursor_position(Some(position));
                self.window.request_redraw();
            }

            WindowEvent::CursorLeft { .. } => {
                self.workspace.set_cursor_position(None);
                self.workspace.clear_title_state();
                self.window.set_title(WINDOW_TITLE);
            }

            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Focused(false) => {
                self.controls.clear_input();
            }

            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.workspace.handle_mouse_input(state);
                self.window.request_redraw();
            }

            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard_input(event, event_loop);
            }

            WindowEvent::Resized(size) => {
                self.renderer.resize(size);
            }

            WindowEvent::RedrawRequested => {
                self.redraw(event_loop);
            }

            _ => {}
        }
    }

    pub fn handle_user_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::VitureImuUpdated => {
                self.window.request_redraw();
            }
        }
    }

    pub fn about_to_wait(&mut self, event_loop: &EventLoopWindowTarget<AppEvent>) {
        if self.controls.is_active() {
            event_loop.set_control_flow(ControlFlow::Poll);
            self.window.request_redraw();
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }

    fn handle_keyboard_input(
        &mut self,
        key_event: KeyEvent,
        event_loop: &EventLoopWindowTarget<AppEvent>,
    ) {
        let mut should_redraw = false;

        if let PhysicalKey::Code(code) = key_event.physical_key {
            if self.controls.handle_key(code, key_event.state) {
                should_redraw = true;
            }
        }

        if key_event.state == ElementState::Pressed && !key_event.repeat {
            if matches!(key_event.physical_key, PhysicalKey::Code(KeyCode::Escape)) {
                event_loop.exit();
                return;
            }

            match key_event.logical_key {
                Key::Named(NamedKey::F11) => {
                    if self.window.fullscreen().is_some() {
                        self.window.set_fullscreen(None);
                    } else {
                        self.window
                            .set_fullscreen(Some(Fullscreen::Borderless(None)));
                    }
                    should_redraw = true;
                }

                Key::Named(NamedKey::Escape) => {
                    event_loop.exit();
                }

                Key::Named(NamedKey::Home) => {
                    self.controls.reset();
                    should_redraw = true;
                }

                Key::Character(ref value) if value.eq_ignore_ascii_case("r") => {
                    self.controls.reset_orientation();
                    should_redraw = true;
                }

                _ => {}
            }
        }

        if should_redraw {
            self.window.request_redraw();
        }
    }

    fn redraw(&mut self, event_loop: &EventLoopWindowTarget<AppEvent>) {
        let now = Instant::now();
        let delta_seconds = (now - self.last_frame).as_secs_f32().min(0.05);
        self.last_frame = now;

        self.controls.update_head(delta_seconds);

        let orientation = self.controls.orientation();
        let camera_position = self.controls.position();

        self.controls.update_navigation(delta_seconds, orientation);

        let frame = self
            .workspace
            .update(&mut self.renderer, orientation, camera_position);

        if let Some(title) = frame.title {
            self.window.set_title(&title);
        }

        match self.renderer.render(orientation, camera_position) {
            Ok(()) => {}

            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.renderer.reconfigure();
            }

            Err(wgpu::SurfaceError::OutOfMemory) => {
                event_loop.exit();
            }

            Err(wgpu::SurfaceError::Timeout) => {}
        }
    }
}
