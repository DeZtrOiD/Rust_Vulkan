// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: пробовать новый язык нужно с нового движка
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#

mod window;
mod vulkan_wr;

use vulkan_wr::app::VulkanApp;
// use vulkan_wr::device::VulkanDevice;

fn main() {
    let app_name = "RUST_POBEDA";
    let height = 720;
    let width = 1280;

    let mut window_ = window::Window::try_new(width, height, app_name, glfw::WindowMode::Windowed)
        .unwrap();
    let aa = VulkanApp::try_new(&window_, app_name).unwrap();

    // Loop until the user closes the window
    while !window_.should_close() {
        window_.process_events();
        aa.run().unwrap();
    }
    // let dev = VulkanDevice::try_new(&aa);
    drop(aa);
    drop(window_);
}
