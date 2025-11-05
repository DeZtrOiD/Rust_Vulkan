// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: пробовать новый язык нужно с нового движка
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#

mod window;
mod vulkan_wr;

use vulkan_wr::app::{VulkanApp};
use vulkan_wr::render_func::{init_app, record_render_commands_app, update_app, shutdown_app, present_frame_app};
use ash::{vk};
// use vulkan_wr::device::VulkanDevice;

fn main() {
    let app_name = "RUST_POBEDA";
    let height = 720;
    let width = 1280;

    let mut window_ = window::Window::try_new(width, height, app_name, glfw::WindowMode::Windowed)
        .unwrap();
    let mut app = VulkanApp::try_new(&window_, app_name).unwrap();

    let image_count = app.get_swapchain_images_count();
    let frame_res = app.get_frame_resources(
        image_count,
        vk::CommandBufferLevel::PRIMARY,
        image_count * 2, // 2 семафора на изображение
        image_count
    ).unwrap();

    app.init(init_app).unwrap();
    app.record_render_graph(record_render_commands_app, &frame_res).unwrap(); // запись команд один раз

    // Loop until the user closes the window
    while !window_.should_close() {
        window_.process_events();
        app.update(update_app).unwrap();
        app.submit_and_present(present_frame_app, &frame_res).unwrap();
    }
    // let dev = VulkanDevice::try_new(&aa);
    app.device_wait_idle().unwrap();
    app.shutdown(shutdown_app).unwrap();

    // drop(frame_res);
    // drop(app);
    // drop(window_);
}
