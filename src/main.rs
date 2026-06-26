mod control_server;
mod dome;
mod dome_config;
mod navigation;
mod orientation;
mod panel;
mod renderer;
mod texture;

use std::{
    sync::Arc,
    time::Instant,
};

use dome_config::{DomeConfig, SharedDomeConfig};
use glam::Vec3;
use navigation::Navigation;
use orientation::{
    keyboard::KeyboardOrientation,
    OrientationSource,
};
use renderer::Renderer;
use winit::{
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey, PhysicalKey},
    window::{Fullscreen, WindowBuilder},
};

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new()
        .expect("Não foi possível criar o event loop");

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("XR Dome")
            .with_inner_size(
                winit::dpi::LogicalSize::new(1280, 720),
            )
            .build(&event_loop)
            .expect("Não foi possível criar a janela"),
    );

    window.set_fullscreen(Some(
        Fullscreen::Borderless(None),
    ));

    let shared_dome_config =
        SharedDomeConfig::new(DomeConfig::default());

    control_server::spawn_control_server(
        shared_dome_config.clone(),
    );

    let initial_config = shared_dome_config.get();

    let (vertices, indices) =
        initial_config.build_mesh();

    /*
     * Painel ultrawide curvo
     */
    let panel_yaw_degrees = 120.0_f32;
    let panel_aspect = 1915.0 / 821.0;

    /*
    * Pequeno offset para o painel ficar na frente da grid,
    * mas ainda seguir a curvatura do domo.
    */
    let panel_surface_offset = 0.03_f32;

    let panel_horizontal_segments = 192_usize;
    let panel_vertical_segments = 48_usize;

    let (panel_vertices, panel_indices) =
        panel::generate_spherical_panel(
            panel_yaw_degrees,
            panel_aspect,
            initial_config.radius,
            panel_surface_offset,
            panel_horizontal_segments,
            panel_vertical_segments,
        );

    let mut renderer =
        pollster::block_on(Renderer::new(
            Arc::clone(&window),
            &vertices,
            &indices,
            &panel_vertices,
            &panel_indices,
            Some("assets/image2.png"),
        ));

    let mut head_orientation = KeyboardOrientation::default();

    /*
     * O observador começa deslocado do centro,
     * mais próximo do painel frontal.
     */

    let mut navigation = Navigation::new(
        Vec3::new(0.0, 0.0, -2.0),
        initial_config.radius,
    );

    let mut last_frame = Instant::now();

    event_loop
        .run(move |event, event_loop| {
            match event {
                Event::WindowEvent {
                    window_id,
                    event,
                } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => {
                            event_loop.exit();
                        }

                        WindowEvent::Focused(false) => {
                            navigation.clear_input();
                            head_orientation.clear_input();
                        }

                        WindowEvent::KeyboardInput {
                            event: key_event,
                            ..
                        } => {
                            /*
                             * Movimento precisa receber tanto
                             * Pressed quanto Released.
                             */

                            if let PhysicalKey::Code(code) =
                                key_event.physical_key
                            {
                                // WASD, Q/E e Shift.
                                navigation.handle_key(
                                    code,
                                    key_event.state,
                                );

                                // Somente as setas.
                                head_orientation.handle_key(
                                    code,
                                    key_event.state,
                                );

                                window.request_redraw();
                            }

                            /*
                             * Atalhos executados uma vez.
                             */

                            if key_event.state
                                == ElementState::Pressed
                                && !key_event.repeat
                            {
                                match key_event.logical_key {
                                    Key::Named(
                                        NamedKey::F11,
                                    ) => {
                                        if window
                                            .fullscreen()
                                            .is_some()
                                        {
                                            window
                                                .set_fullscreen(None);
                                        } else {
                                            window.set_fullscreen(
                                                Some(
                                                    Fullscreen::Borderless(
                                                        None,
                                                    ),
                                                ),
                                            );
                                        }
                                    }

                                    Key::Named(
                                        NamedKey::Escape,
                                    ) => {
                                        event_loop.exit();
                                    }

                                    Key::Named(
                                        NamedKey::Home,
                                    ) => {
                                        head_orientation.reset();
                                        navigation.reset();
                                    }

                                    Key::Character(ref value)
                                        if value
                                            .eq_ignore_ascii_case(
                                                "r",
                                            ) =>
                                    {
                                        head_orientation.reset();
                                    }

                                    _ => {}
                                }
                            }
                        }

                        WindowEvent::Resized(size) => {
                            renderer.resize(size);
                        }

                        WindowEvent::RedrawRequested => {
                            let now = Instant::now();

                            let delta_seconds =
                                (now - last_frame)
                                    .as_secs_f32()
                                    .min(0.05);

                            last_frame = now;

                            head_orientation.update(delta_seconds);

                            let orientation =
                                head_orientation.orientation();

                            navigation.update(
                                delta_seconds,
                                orientation,
                            );

                            match renderer.render(
                                orientation,
                                navigation.position(),
                            ) {
                                Ok(()) => {}

                                Err(
                                    wgpu::SurfaceError::Lost
                                    | wgpu::SurfaceError::Outdated,
                                ) => {
                                    renderer.reconfigure();
                                }

                                Err(
                                    wgpu::SurfaceError::OutOfMemory,
                                ) => {
                                    event_loop.exit();
                                }

                                Err(
                                    wgpu::SurfaceError::Timeout,
                                ) => {}
                            }
                        }

                        _ => {}
                    }
                }

                Event::AboutToWait => {
                    let mut needs_redraw = false;

                    if shared_dome_config.take_dirty() {
                        let config = shared_dome_config.get();

                        navigation.set_dome_radius(config.radius);

                        let (vertices, indices) = config.build_mesh();

                        renderer.update_dome_mesh(
                            &vertices,
                            &indices,
                        );

                        let (panel_vertices, panel_indices) =
                            panel::generate_spherical_panel(
                                panel_yaw_degrees,
                                panel_aspect,
                                config.radius,
                                panel_surface_offset,
                                panel_horizontal_segments,
                                panel_vertical_segments,
                            );

                        renderer.update_panel_mesh(
                            &panel_vertices,
                            &panel_indices,
                        );

                        println!(
                            "dome updated: yaw={} pitch={}..{} radius={} segments={}x{}",
                            config.yaw_degrees,
                            config.min_pitch_degrees,
                            config.max_pitch_degrees,
                            config.radius,
                            config.horizontal_segments,
                            config.vertical_segments,
                        );

                        needs_redraw = true;
                    }

                    let active_motion =
                        navigation.is_moving()
                            || head_orientation.is_rotating();

                    if active_motion {
                        event_loop.set_control_flow(ControlFlow::Poll);
                        needs_redraw = true;
                    } else {
                        event_loop.set_control_flow(ControlFlow::Wait);
                    }

                    if needs_redraw {
                        window.request_redraw();
                    }
                }

                _ => {}
            }
        })
        .expect("Erro no event loop");
}