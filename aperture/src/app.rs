use winit::event::{DeviceEvent, ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ControlFlow;

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
    let (event_loop, renderer) = Renderer::new(config.title, config.width, config.height);
    let input_state = InputState::default();

    let mut app = App {
        renderer,
        input_state,
    };

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
                event: WindowEvent::Resized(size),
                ..
            } => {
                if size.width > 0 && size.height > 0 {
                    // app.resize();
                }
            }
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        device_id: _,
                        state,
                        button,
                        ..
                    },
                ..
            } => match button {
                MouseButton::Left => {
                    app.input_state.mouse_left_down = state == ElementState::Pressed
                }
                MouseButton::Right => {
                    app.input_state.mouse_right_down = state == ElementState::Pressed
                }
                _ => {}
            },
            Event::DeviceEvent {
                event: DeviceEvent::MouseWheel { delta },
                ..
            } => {
                app.input_state.wheel_delta = match delta {
                    MouseScrollDelta::LineDelta(_, delta_y) => Some(delta_y),
                    MouseScrollDelta::PixelDelta(delta) => Some(delta.y as f32),
                };
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                app.input_state.position_delta = Some([delta.0 as f32, delta.1 as f32]);
            }
            Event::MainEventsCleared => {
                app.render();
                app.input_state.tick();
            }
            _ => {}
        }
    });
}
