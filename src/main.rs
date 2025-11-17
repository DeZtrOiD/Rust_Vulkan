// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: пробовать новый язык нужно с нового движка
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#

mod window;
mod vulkan_wr;
mod scenes;

use vulkan_wr::app::{VulkanApp};

use scenes::sphere::{
    frame_resources::get_frame_resources,
    init::init_app,
    update::update_app,
    render_frame::render_frame_app,
    shutdown::shutdown_app,
};


fn main() {
    let app_name = "RUST_POBEDA";
    let height = 720;
    let width = 1280;
    let window_ = window::Window::try_new(width, height, app_name, glfw::WindowMode::Windowed)
        .unwrap();
    let mut app = VulkanApp::try_new(window_, app_name).unwrap();

    let image_count = app.get_swapchain_images_count();
    let mut frame_res = app.get_frame_resources(
        image_count,
        get_frame_resources
    ).unwrap();

    app.init(init_app, &mut frame_res).unwrap();

    // Loop until the user closes the window
    while !app.should_close() {
        app.update(update_app, &mut frame_res).unwrap();
        app.render(render_frame_app, &mut frame_res).unwrap();
    }

    app.device_wait_idle().unwrap();
    app.shutdown(shutdown_app,  &mut frame_res).unwrap();

}
