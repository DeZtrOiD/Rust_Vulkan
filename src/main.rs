// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: пробовать новый язык нужно с нового движка
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#

mod window;
mod vulkan_wr;
mod scenes;

use vulkan_wr::app::{VulkanApp};

#[cfg(not(feature = "scene3"))]
use scenes::common::{
    frame_resources::FrameResources,
    init::init_app,
    render_frame::render_frame_app,
    shutdown::shutdown_app,
    update::update_app,
};

#[cfg(feature = "scene3")]
use scenes::dynamic::{
    frame_resources::FrameResources,
    init::init_app,
    render_frame::render_frame_app,
    shutdown::shutdown_app,
    update::update_app,
};


#[cfg(feature = "scene1")]
use scenes::sphere::{
    frame_resources::ImguiFrameResourcesSphere,
    update::ResourcesSphere,
};

#[cfg(feature = "scene2")]
use crate::scenes::lighting::frame_resources::ImguiFrameResourcesLight;
#[cfg(feature = "scene2")]
use crate::scenes::lighting::update::ResourcesLight;
#[cfg(feature = "scene3")]
use crate::scenes::shadows::frame_resources::ImguiFrameResourcesShadows;
#[cfg(feature = "scene3")]
use crate::scenes::shadows::update::ResourcesShadows;

fn main() {
    let app_name = "RUST_POBEDA";
    let height = 720;
    let width = 1280;
    let window_ = window::Window::try_new(width, height, app_name, glfw::WindowMode::Windowed)
        .unwrap();
    let mut app = VulkanApp::try_new(window_, app_name).unwrap();

    let image_count = app.get_swapchain_images_count();
    #[cfg(feature = "scene1")]
    let mut frame_res = app.get_frame_resources::<FrameResources<ImguiFrameResourcesSphere>>
    (
        image_count
    ).unwrap();

    #[cfg(feature = "scene2")]
    let mut frame_res = app.get_frame_resources::<FrameResources<ImguiFrameResourcesLight>>
    (
        image_count
    ).unwrap();

    #[cfg(feature = "scene3")]
    let mut frame_res = app.get_frame_resources::<FrameResources<ImguiFrameResourcesShadows>>
    (
        image_count
    ).unwrap();

    app.init(init_app, &mut frame_res).unwrap();

    // Loop until the user closes the window
    while !app.should_close() {
        app.update(
            #[cfg(feature = "scene1")]
            update_app::<ImguiFrameResourcesSphere, ResourcesSphere>,
            #[cfg(feature = "scene2")]
            update_app::<ImguiFrameResourcesLight, ResourcesLight>,
            #[cfg(feature = "scene3")]
            update_app::<ImguiFrameResourcesShadows, ResourcesShadows>,
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
