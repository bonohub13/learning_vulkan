mod hello_triangle_application;

use hello_triangle_application::HelloTriangleApplication;

fn main() {
    let mut app = HelloTriangleApplication::new();

    app.run();
}
