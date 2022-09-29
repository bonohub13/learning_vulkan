mod _pipeline {
    use ash::vk;

    pub fn create_descriptor_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
        // Descriptor set layout
        let ubo_layout_bindings = [vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build()];
        let layout_info =
            vk::DescriptorSetLayoutCreateInfo::builder().bindings(&ubo_layout_bindings);

        let descriptor_set_layout = unsafe {
            device
                .create_descriptor_set_layout(&layout_info, None)
                .expect("failed to create descriptor set layout!")
        };

        descriptor_set_layout
    }

    pub fn create_shader_module(device: &ash::Device, code: &Vec<u8>) -> vk::ShaderModule {
        // FIXED: Major bug when creating shader module!
        let create_info = vk::ShaderModuleCreateInfo {
            code_size: code.len(),
            p_code: code.as_ptr() as *const u32,
            ..Default::default()
        };

        unsafe {
            device
                .create_shader_module(&create_info, None)
                .expect("failed to create shader module!")
        }
    }
}

pub use _pipeline::{create_descriptor_set_layout, create_shader_module};
