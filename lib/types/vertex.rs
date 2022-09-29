#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex2D {
    // Vertex data
    pub pos: [f32; 2],
    pub color: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VertexWithTexture2D {
    // Texture coordinates
    pub pos: [f32; 2],       // position for each point
    pub color: [f32; 3],     // color (R, G, B)
    pub tex_coord: [f32; 2], // coordinate for texture
}

impl Vertex2D {
    #[inline]
    pub fn new(pos: [f32; 2], color: [f32; 3]) -> Self {
        Self { pos, color }
    }

    #[inline]
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

    #[inline]
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

impl VertexWithTexture2D {
    #[inline]
    pub fn new(pos: [f32; 2], color: [f32; 3], texture_coordinate: [f32; 2]) -> Self {
        Self {
            pos,
            color,
            tex_coord: texture_coordinate,
        }
    }

    #[inline]
    pub fn get_binding_description() -> [ash::vk::VertexInputBindingDescription; 1] {
        use ash::vk;
        use std::mem::size_of;

        [vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()]
    }

    #[inline]
    pub fn get_attribute_descriptions() -> [ash::vk::VertexInputAttributeDescription; 3] {
        use ash::vk;
        use memoffset::offset_of;

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
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Self, tex_coord) as u32)
                .build(),
        ]
    }
}
