use crate::render::Renderer;
use crate::state::InputState;

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
            title: "Aperture Renderer".to_string(),
        }
    }
}

pub struct App {
    pub renderer: Renderer,
    pub input_state: InputState,
}

impl App {
    pub fn render(&mut self) {
        self.renderer.render();
    }
}

pub fn run_app(config: AppConfig) {
    let renderer = Renderer::new(config.title, config.width, config.height);
    let input_state = InputState::default();

    let _app = App {
        renderer,
        input_state,
    };
}
