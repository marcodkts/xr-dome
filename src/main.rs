mod app;
mod app_event;
mod controls;
mod dome;
mod dome_config;
mod integrations;
mod navigation;
mod orientation;
mod ray;
mod renderer;
mod settings;
mod surface;
mod texture;
mod workspace;

use std::sync::Arc;

use app::{App, WINDOW_TITLE};
use app_event::AppEvent;
use winit::{
    event::Event,
    event_loop::EventLoopBuilder,
    window::{Fullscreen, WindowBuilder},
};

fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let event_loop = EventLoopBuilder::<AppEvent>::with_user_event()
        .build()
        .expect("Não foi possível criar o event loop");
    let event_proxy = event_loop.create_proxy();

    let window = Arc::new(
        WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .build(&event_loop)
            .expect("Não foi possível criar a janela"),
    );

    window.set_fullscreen(Some(Fullscreen::Borderless(None)));

    let mut app = App::new(Arc::clone(&window), event_proxy);

    event_loop
        .run(move |event, event_loop| match event {
            Event::WindowEvent { window_id, event } if window_id == app.window_id() => {
                app.handle_window_event(event, event_loop);
            }

            Event::UserEvent(event) => {
                app.handle_user_event(event);
            }

            Event::AboutToWait => {
                app.about_to_wait(event_loop);
            }

            _ => {}
        })
        .expect("Erro no event loop");
}
