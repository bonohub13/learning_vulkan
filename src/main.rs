use vk_utils::constants::*;

use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Vulkan tutorial with ash")
        .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
        .with_min_inner_size(PhysicalSize::new(MINIMAL_WIDTH, MINIMAL_HEIGHT))
        .build(&event_loop)
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            _ => (),
        }
    });
}
