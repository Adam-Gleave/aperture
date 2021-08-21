mod app;
mod render;
mod state;
mod vulkan;

use app::*;

fn main() {
    run_app(AppConfig {
        title: "Aperture Renderer".to_string(),
        width: 1560,
        height: 980,
    });
}
