// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: 
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#

mod window;
mod vulkan_wr;

use vulkan_wr::instance::VulkanInstance;
// use vulkan_wr::device::VulkanDevice;

fn main() {

    let mut window_ = window::Window::try_new(512, 512, "RUST_POBEDA", glfw::WindowMode::Windowed)
        .unwrap();
    let aa = VulkanInstance::try_new(&window_).unwrap();

    // Loop until the user closes the window
    while !window_.should_close() {
        window_.process_events();
    }
    // let dev = VulkanDevice::try_new(&aa);
}