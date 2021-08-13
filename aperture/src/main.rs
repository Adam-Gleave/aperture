mod app;
mod render;
mod state;
mod vulkan;
mod world;

use app::*;

#[tokio::main]
async fn main() {
    run_app(AppConfig {
        title: "Aperture Renderer".to_string(),
        width: 1560,
        height: 980,
    });
}
