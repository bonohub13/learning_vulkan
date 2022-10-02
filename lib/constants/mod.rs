mod _constants {
    use crate::{VkDeviceExtension, VkValidationInfo};
    use ash::vk::make_api_version;

    pub const WIDTH: u32 = 800;
    pub const HEIGHT: u32 = 600;
    pub const MINIMAL_WIDTH: u32 = 100;
    pub const MINIMAL_HEIGHT: u32 = 100;
    pub const MAXIMUM_WIDTH: u32 = 1920;
    pub const MAXIMUM_HEIGHT: u32 = 1080;

    pub const ENGINE_NAME: &str = "No Engine";
    pub const ENGINE_VERSION: u32 = make_api_version(0, 1, 0, 0);

    pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

    pub const VK_VALIDATION_LAYER_NAMES: VkValidationInfo = VkValidationInfo {
        is_enable: true,
        required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
    };
    pub const VK_DEVICE_EXTENSIONS: VkDeviceExtension = VkDeviceExtension {
        names: ["VK_KHR_swapchain"],
    };
}

pub use _constants::{
    ENGINE_NAME,    // engine name
    ENGINE_VERSION, // engine version
};
pub use _constants::{HEIGHT, WIDTH};
// minimal/maximum window size
pub use _constants::{MAXIMUM_HEIGHT, MAXIMUM_WIDTH, MINIMAL_HEIGHT, MINIMAL_WIDTH};

pub use _constants::MAX_FRAMES_IN_FLIGHT;

pub use _constants::{
    VK_DEVICE_EXTENSIONS,      // vulkan device extensions
    VK_VALIDATION_LAYER_NAMES, // vulkan validation layers
};

pub mod hello_triangle {
    // module for specifically for hello_triangle
    pub const APPLICATION_NAME: &str = // app name
        "Hello Triangle";
    pub const APPLICATION_VERSION: u32 = // app version
        ash::vk::make_api_version(0, 1, 0, 0);

    pub const TRIANGLE_VERTEX: [crate::types::Vertex2D; 3] = [
        crate::types::Vertex2D {
            pos: [0.0, -0.5],
            color: [1.0, 0.0, 0.0],
        },
        crate::types::Vertex2D {
            pos: [0.5, 0.5],
            color: [0.0, 1.0, 0.0],
        },
        crate::types::Vertex2D {
            pos: [-0.5, 0.5],
            color: [0.0, 0.0, 1.0],
        },
    ];

    pub const VERTICES: [crate::types::Vertex2D; 4] = [
        crate::types::Vertex2D {
            pos: [-0.5, -0.5],
            color: [1.0, 0.0, 0.0],
        },
        crate::types::Vertex2D {
            pos: [0.5, -0.5],
            color: [0.0, 1.0, 0.0],
        },
        crate::types::Vertex2D {
            pos: [0.5, 0.5],
            color: [0.0, 0.0, 1.0],
        },
        crate::types::Vertex2D {
            pos: [-0.5, 0.5],
            color: [1.0, 1.0, 1.0],
        },
    ];

    pub const INDICES: [u32; 6] = [0, 1, 2, 2, 3, 0];
}

pub mod texture {
    pub const APPLICATION_NAME: &str = // app name
        "Texture";
    pub const APPLICATION_VERSION: u32 = // app version
        ash::vk::make_api_version(0, 2, 1, 0);

    pub const TEXTURE_PATH: &'static str = "assets/texture.jpg";

    pub const INDICES: [u32; 12] = [0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4];

    pub const SQUARE_VERTICES: [crate::types::VertexWithTexture2D; 4] = [
        crate::types::VertexWithTexture2D {
            pos: [-0.5, -0.5],
            color: [1.0, 0.0, 0.0],
            tex_coord: [0.0, 1.0],
        },
        crate::types::VertexWithTexture2D {
            pos: [0.5, -0.5],
            color: [0.0, 1.0, 0.0],
            tex_coord: [0.0, 0.0],
        },
        crate::types::VertexWithTexture2D {
            pos: [0.5, 0.5],
            color: [0.0, 0.0, 1.0],
            tex_coord: [1.0, 0.0],
        },
        crate::types::VertexWithTexture2D {
            pos: [-0.5, 0.5],
            color: [1.0, 1.0, 1.0],
            tex_coord: [1.0, 1.0],
        },
    ];

    pub const VERTICES: [crate::types::VertexWithTexture3D; 8] = [
        crate::types::VertexWithTexture3D {
            pos: [-0.5, -0.5, 0.0, 1.0],
            color: [1.0, 0.0, 0.0],
            tex_coord: [0.0, 0.0],
        },
        crate::types::VertexWithTexture3D {
            pos: [0.5, -0.5, 0.0, 1.0],
            color: [0.0, 1.0, 0.0],
            tex_coord: [1.0, 0.0],
        },
        crate::types::VertexWithTexture3D {
            pos: [0.5, 0.5, 0.0, 1.0],
            color: [0.0, 0.0, 1.0],
            tex_coord: [1.0, 1.0],
        },
        crate::types::VertexWithTexture3D {
            pos: [-0.5, 0.5, 0.0, 1.0],
            color: [1.0, 1.0, 1.0],
            tex_coord: [0.0, 1.0],
        },
        crate::types::VertexWithTexture3D {
            pos: [-0.5, -0.5, -0.5, 1.0],
            color: [1.0, 0.0, 0.0],
            tex_coord: [0.0, 0.0],
        },
        crate::types::VertexWithTexture3D {
            pos: [0.5, -0.5, -0.5, 1.0],
            color: [0.0, 1.0, 0.0],
            tex_coord: [1.0, 0.0],
        },
        crate::types::VertexWithTexture3D {
            pos: [0.5, 0.5, -0.5, 1.0],
            color: [0.0, 0.0, 1.0],
            tex_coord: [1.0, 1.0],
        },
        crate::types::VertexWithTexture3D {
            pos: [-0.5, 0.5, -0.5, 1.0],
            color: [1.0, 1.0, 1.0],
            tex_coord: [0.0, 1.0],
        },
    ];
}

pub mod model {
    pub const APPLICATION_NAME: &str = // app name
        "Loading models";
    pub const APPLICATION_VERSION: u32 = // app version
        ash::vk::make_api_version(0, 3, 1, 0);

    pub const MODEL_PATH: &'static str = "assets/viking_room.obj";
    pub const TEXTURE_PATH: &'static str = "assets/viking_room.png";
}
