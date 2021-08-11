use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

use crate::render::Renderer;
use crate::world::World;

use std::path::Path;

pub struct AppConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            width: 1560,
            height: 980,
            title: "Vulkan Renderer".to_string(),
        }
    }
}

pub struct App {
    pub world: World,
    pub renderer: Renderer,
}

impl App {
    pub fn resize(&mut self) {
        self.renderer.notify_resized();
    }

    pub fn load_gltf<P: AsRef<Path>>(&mut self, path: P) {
        self.world.load_gltf(path);
    }

    pub fn render(&mut self) {
        self.renderer.render(&self.world);
    }
}

pub fn run_app(config: AppConfig) {
    let (renderer, event_loop) = Renderer::new(config.title, config.width, config.height);
    let world = World;

    let mut app = App { renderer, world };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                app.resize();
            }
            Event::MainEventsCleared => {
                app.render();
            }
            _ => {}
        }
    });
}
