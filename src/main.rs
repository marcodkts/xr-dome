mod control_server;
mod dome;
mod dome_config;
mod orientation;
mod panel;
mod renderer;
mod texture;

use std::sync::Arc;

use dome_config::{DomeConfig, SharedDomeConfig};
use orientation::{mouse::MouseOrientation, OrientationSource};
use renderer::Renderer;
use winit::{
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::WindowBuilder,
};

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new()
        .expect("Não foi possível criar o event loop");

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("XR Dome")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .build(&event_loop)
            .expect("Não foi possível criar a janela"),
    );

    let shared_dome_config = SharedDomeConfig::new(DomeConfig::default());

    control_server::spawn_control_server(shared_dome_config.clone());

    let initial_config = shared_dome_config.get();
    let (vertices, indices) = initial_config.build_mesh();

    let (panel_vertices, panel_indices) = panel::generate_curved_panel(
        2.8,
        1.575,
        3.4,
        96,
        12,
    );

    let mut renderer = pollster::block_on(Renderer::new(
        Arc::clone(&window),
        &vertices,
        &indices,
        &panel_vertices,
        &panel_indices,
        Some("assets/PM5644.png"),
    ));

    let mut mouse = MouseOrientation::default();

    event_loop
        .run(move |event, event_loop| {
            event_loop.set_control_flow(ControlFlow::Poll);

            match event {
                Event::DeviceEvent { event, .. } => {
                    mouse.handle_device_event(&event);
                }

                Event::WindowEvent { window_id, event }
                    if window_id == window.id() =>
                {
                    mouse.handle_window_event(&event);

                    match event {
                        WindowEvent::CloseRequested => {
                            event_loop.exit();
                        }

                        WindowEvent::KeyboardInput {
                            event: key_event,
                            ..
                        } => {
                            if key_event.state == ElementState::Pressed
                                && !key_event.repeat
                            {
                                match key_event.logical_key {
                                    Key::Named(NamedKey::Escape) => {
                                        event_loop.exit();
                                    }

                                    Key::Named(NamedKey::ArrowLeft) => {
                                        mouse.rotate_by_keyboard(
                                            5.0_f32.to_radians(),
                                            0.0,
                                        );
                                    }

                                    Key::Named(NamedKey::ArrowRight) => {
                                        mouse.rotate_by_keyboard(
                                            -5.0_f32.to_radians(),
                                            0.0,
                                        );
                                    }

                                    Key::Named(NamedKey::ArrowUp) => {
                                        mouse.rotate_by_keyboard(
                                            0.0,
                                            5.0_f32.to_radians(),
                                        );
                                    }

                                    Key::Named(NamedKey::ArrowDown) => {
                                        mouse.rotate_by_keyboard(
                                            0.0,
                                            -5.0_f32.to_radians(),
                                        );
                                    }

                                    Key::Character(ref value)
                                        if value.eq_ignore_ascii_case("r") =>
                                    {
                                        mouse.reset();
                                    }

                                    _ => {}
                                }
                            }
                        }

                        WindowEvent::Resized(size) => {
                            renderer.resize(size);
                        }

                        WindowEvent::RedrawRequested => {
                            match renderer.render(mouse.orientation()) {
                                Ok(()) => {}

                                Err(
                                    wgpu::SurfaceError::Lost
                                    | wgpu::SurfaceError::Outdated,
                                ) => {
                                    renderer.reconfigure();
                                }

                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    event_loop.exit();
                                }

                                Err(wgpu::SurfaceError::Timeout) => {}
                            }
                        }

                        _ => {}
                    }
                }

                Event::AboutToWait => {
                    if shared_dome_config.take_dirty() {
                        let config = shared_dome_config.get();
                        let (vertices, indices) = config.build_mesh();

                        renderer.update_dome_mesh(&vertices, &indices);

                        println!(
                            "dome updated: yaw={} pitch={}..{} radius={} segments={}x{}",
                            config.yaw_degrees,
                            config.min_pitch_degrees,
                            config.max_pitch_degrees,
                            config.radius,
                            config.horizontal_segments,
                            config.vertical_segments,
                        );
                    }

                    window.request_redraw();
                }

                _ => {}
            }
        })
        .expect("Erro no event loop");
}