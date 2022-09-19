mod hello_triangle_application;

use vk_utils::constants::*;

use hello_triangle_application::HelloTriangleTriangle;

use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Vulkan")
        .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
        .with_min_inner_size(PhysicalSize::new(MINIMAL_WIDTH, MINIMAL_HEIGHT))
        .with_max_inner_size(PhysicalSize::new(MAXIMUM_WIDTH, MAXIMUM_HEIGHT))
        .build(&event_loop)
        .unwrap();
    let mut app = HelloTriangleTriangle::new(&window);

    // Application loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            // Press the "erase window" button or press the escape key to kill
            // app
            Event::WindowEvent {
                event:
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            // Main event for app
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_window_id) => {
                // TODO Draw frame here!
                app.draw_frame();
            }
            Event::LoopDestroyed => {
                app.wait_for_device_idle();
            }
            // Some other random events
            _ => (),
        }
    });
}
