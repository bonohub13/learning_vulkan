pub struct VkSurface {
    pub surface_loader: ash::extensions::khr::Surface,
    pub surface: ash::vk::SurfaceKHR,
}

impl VkSurface {
    pub fn new(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &winit::window::Window,
    ) -> Self {
        use ash::extensions::khr::Surface;

        let surface = unsafe {
            ash_window::create_surface(entry, instance, window, None)
                .expect("Failed to create window surface")
        };
        let surface_loader = Surface::new(entry, instance);

        Self {
            surface_loader,
            surface,
        }
    }
}
