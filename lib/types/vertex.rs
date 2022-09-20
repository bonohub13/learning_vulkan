#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex2D {
    // Vertex data
    pub pos: [f32; 2],
    pub color: [f32; 3],
}

impl Vertex2D {
    pub fn new(pos: [f32; 2], color: [f32; 3]) -> Self {
        Self { pos, color }
    }

    pub fn get_binding_description() -> [ash::vk::VertexInputBindingDescription; 1] {
        use ash::vk;
        use std::mem::size_of;

        // Binding descriptions
        [vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()]
    }

    pub fn get_attribute_descriptions() -> [ash::vk::VertexInputAttributeDescription; 2] {
        use ash::vk;
        use memoffset::offset_of;

        // Binding descriptions
        [
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Self, pos) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Self, color) as u32)
                .build(),
        ]
    }
}
