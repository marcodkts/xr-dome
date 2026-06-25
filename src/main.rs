mod dome;
mod orientation;
mod renderer;

use std::sync::Arc;

use orientation::{
    mouse::MouseOrientation,
    OrientationSource,
};
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

    let (vertices, indices) = dome::generate_dome(
        128,   // quantidade de segmentos
        3.0,   // raio
        2.2,   // altura
        180.0, // arco horizontal
    );

    let mut renderer = pollster::block_on(Renderer::new(
        Arc::clone(&window),
        &vertices,
        &indices,
    ));

    let mut mouse = MouseOrientation::default();

    event_loop
        .run(move |event, event_loop| {
            event_loop.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent {
                    window_id,
                    event,
                } if window_id == window.id() => {
                    mouse.handle_event(&event);

                    match event {
                        WindowEvent::CloseRequested => {
                            event_loop.exit();
                        }

                        WindowEvent::KeyboardInput {
                            event: key_event,
                            ..
                        } => {
                            if key_event.state == ElementState::Pressed
                                && key_event.logical_key
                                    == Key::Named(NamedKey::Escape)
                            {
                                event_loop.exit();
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
                    window.request_redraw();
                }

                _ => {}
            }
        })
        .expect("Erro no event loop");
}