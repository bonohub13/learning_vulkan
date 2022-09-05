pub mod constants;
pub mod tools;

pub mod device;

// Types and objects made just to make things work will be under here!

pub struct VkValidationInfo {
    pub is_enable: bool,
    pub required_validation_layers: [&'static str; 1],
}

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn new(graphics_family: Option<u32>) -> Self {
        Self { graphics_family }
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some()
    }
}
