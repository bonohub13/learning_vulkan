pub mod constants;
pub mod tools;

pub struct VkValidationInfo {
    pub is_enable: bool,
    pub required_validation_layer: [&'static str; 1],
}
