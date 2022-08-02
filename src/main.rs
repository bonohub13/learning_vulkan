mod hello_triangle_application;

use hello_triangle_application::VkAppBase;

fn main() {
    let app = VkAppBase::new();

    app.render_loop(|| {});
}
