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
    let mut tick_counter = vk_utils::fps::FPSLimiter::new();
    let mut app = HelloTriangleTriangle::new(&window);

    // Application loop
    event_loop.run(move |event, _, control_flow| {
        app.wait_for_device_idle();
        *control_flow = ControlFlow::Poll;

        match event {
            // Press the "erase window" button or press the escape key to kill
            // app
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    app.wait_for_device_idle();
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::KeyboardInput { input, .. } => match input {
                    KeyboardInput {
                        virtual_keycode,
                        state,
                        ..
                    } => match (virtual_keycode, state) {
                        (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                            app.wait_for_device_idle();
                            *control_flow = ControlFlow::Exit;
                        }
                        _ => {}
                    },
                },
                WindowEvent::Resized(_new_size) => {
                    app.wait_for_device_idle();
                    app.resize_framebuffer();
                }
                _ => {}
            },
            // Main event for app
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_window_id) => {
                let delta_time = tick_counter.delta_time();

                app.draw_frame(delta_time);
                tick_counter.tick_frame();
            }
            Event::LoopDestroyed => {
                app.wait_for_device_idle();
            }
            // Some other random events
            _ => (),
        }
    });
}
