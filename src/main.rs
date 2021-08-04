use vulkano::{instance::Instance, Version};
use vulkano_win::VkSurfaceBuild;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let ev_loop = EventLoop::new();
    let instance = {
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, Version::V1_1, &extensions, None)
            .unwrap()
    };

    let surface = WindowBuilder::new()
        .build_vk_surface(&ev_loop, instance.clone())
        .unwrap();

    ev_loop.run(move |ev, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match ev {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            },
            Event::MainEventsCleared => {
                surface.window().request_redraw();
            },
            _ => (),
        }
    });
}
