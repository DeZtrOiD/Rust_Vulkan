// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: пробовать новый язык нужно с нового движка
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#

mod window;
mod vulkan_wr;
mod scenes;

use vulkan_wr::app::{VulkanApp};

use scenes::common::{
    frame_resources::FrameResources,
    init::init_app,
    render_frame::render_frame_app,
    shutdown::shutdown_app,
    update::update_app,
};

use scenes::sphere::{
    frame_resources::ImguiFrameResources,
    update::ResourcesSphere,
};

fn main() {
    let app_name = "RUST_POBEDA";
    let height = 720;
    let width = 1280;
    let window_ = window::Window::try_new(width, height, app_name, glfw::WindowMode::Windowed)
        .unwrap();
    let mut app = VulkanApp::try_new(window_, app_name).unwrap();

    let image_count = app.get_swapchain_images_count();
    let mut frame_res: FrameResources<ImguiFrameResources> = app.get_frame_resources::<FrameResources<ImguiFrameResources>>(
        image_count
    ).unwrap();

    app.init(init_app, &mut frame_res).unwrap();

    // Loop until the user closes the window
    while !app.should_close() {
        app.update(
            update_app::<ImguiFrameResources, ResourcesSphere>,
            &mut frame_res
        ).unwrap();
        app.render(render_frame_app, &mut frame_res).unwrap();

        let (width, height) = app.window.get_width_height();
        if width == 0 || height == 0 {
            // Окно минимизировано или скрыто, ждем восстановления
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }

    }

    app.device_wait_idle().unwrap();
    app.shutdown(shutdown_app,  &mut frame_res).unwrap();

}
