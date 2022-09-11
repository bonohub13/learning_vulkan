# Learning Vulkan via Rust + Ash

Learning Vulkan via Ash sample code and [vulkan-tutorial](https://vulkan-tutorial.com/Introduction)

## Chapters
- [chapter 0](https://github.com/bonohub13/learning_vulkan/tree/chapter_0)
    - [Base Code](https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Base_code)
- [chapter 1](https://github.com/bonohub13/learning_vulkan/tree/chapter_1)
    - [Instance](https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Instance)
- [chapter 2](https://github.com/bonohub13/learning_vulkan/tree/chapter_2)
    - [Validation Layers](https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Validation_layers)
- [chapter 3](https://github.com/bonohub13/learning_vulkan/tree/chapter_3)
    - [Physical devices and queue families](https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Physical_devices_and_queue_families)
- [chapter 4](https://github.com/bonohub13/learning_vulkan/tree/chapter_4)
    - [Logical device and queues](https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Logical_device_and_queues)
- [chapter 5](https://github.com/bonohub13/learning_vulkan/tree/chapter_5)
    - [Logical device and queues](https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Window_surface)
- [chapter 6](https://github.com/bonohub13/learning_vulkan/tree/chapter_6)
    - [Swap chain](https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Swap_chain)
- [chapter 7](https://github.com/bonohub13/learning_vulkan/tree/chapter_7)
    - [Image views](https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Image_views)
- [chapter 8](https://github.com/bonohub13/learning_vulkan/tree/chapter_8)
    - [Introduction](https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Introduction)

## Building
For building the code in this project, it is recommended to use docker with `make docker-build`. \
When building the codes with docker, it is highly recommended to use [docker-rootless](https://docs.docker.com/engine/security/rootless/). \
If you build using standard docker (not rootless docker), the built binary might end up
with root user ownership. (which sucks BTW when you want to build, run, remove the file)
