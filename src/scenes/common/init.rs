
use crate::vulkan_wr::{app::SceneResources};

use super::{
    frame_resources::{FrameResources},
    renderable_object::RenderObjectEnum,
    // uniform::Uniforms
};
use super::super::super::vulkan_wr::{
    ImGui_wr::{VulkanImgui, ImguiResources},
    app::VulkanApp,
    render_pass::{subpass::SubpassConfigBuilder, pass::VulkanRenderPass},
    renderable_traits::InitObject,
    renderable_traits::InitFrameResources,
};
use ash::vk;
// use imgui::internal::RawWrapper;
// use super::objects::{InitSphereObject, SphereObject};
use crate::scenes::sphere::objects::SphereObject;


pub fn init_app<R: ImguiResources + Default>(app: &mut VulkanApp, resources: &mut FrameResources<R>) -> Result<(), &'static str> {

    // 1. Render pass
    // vk::AttachmentDescription метаинфа одного вложения в рендерпасе
    // * `format` - формат пикселей вложения (должен соответствовать формату изображения)
    // * `samples` - количество сэмплов для мультисэмплинга (обычно TYPE_1 для отсутствия мультисэмплинга)
    // * `load_op` - операция при начале рендер-пасса (CLEAR, LOAD или DONT_CARE)
    // * `store_op` - операция при завершении рендер-пасса (STORE, DONT_CARE)
    // * `stencil_load_op` - операция загрузки для трафаретного буфера
    // * `stencil_store_op` - операция сохранения для трафаретного буфера
    // * `initial_layout` - начальный layout изображения перед рендер-пассом
    // * `final_layout` - конечный layout изображения после рендер-пасса
    let color_attachment = vk::AttachmentDescription {
        format: app.swapchain.color_format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,  // операция при начале рендер-пасса (CLEAR, LOAD, DONT_CARE)
        store_op: vk::AttachmentStoreOp::STORE, // в конце (STORE, DONT_CARE)
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        ..Default::default()
    };

    // Depth attachment
    let depth_attachment = vk::AttachmentDescription {
        format: app.swapchain.depth_format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::DONT_CARE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        ..Default::default()
    };

    // vk::AttachmentReference указывает, какое вложение используется в сабпассе
    // AttachmentDescription - метаинформация для всео рендерпасса
    // reference - информация для сабпасса, описывает состояние в нем и то какие используются
    let att_arr = vec![
        vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        },
        vk::AttachmentReference {
            attachment: 1, // Индекс depth attachment
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        }
    ];


    let subpass = vec![SubpassConfigBuilder::new()
        .bind_point(vk::PipelineBindPoint::GRAPHICS)
        .add_color_attachment(att_arr[0])
        .add_attachment(color_attachment)
        .add_depth_stencil(att_arr[1])
        .add_attachment(depth_attachment)
        .build()];

    let render_pass = VulkanRenderPass::try_new(
        subpass,
        vec![],  // вектор SubpassDependency между файлами
        &app.core._logical_device
    )?;

    resources.render_pass = Some(render_pass);


    resources.vec_objects.push(RenderObjectEnum::Sphere(SphereObject::init(
            app,
            &mut InitFrameResources {
                render_pass: Some(resources.render_pass.as_ref().unwrap()),
                upload_cmd: Some(&resources.vec_cmd_primary[0]),
                fence: Some(&resources.vec_fence[0]),
            }
        )?)
    );
    // ----- IMGUI ------  СНОВА СНОВА ПОСЛЕДНИЙ 
    resources.vec_objects.push(RenderObjectEnum::ImGui(VulkanImgui::<R>::init(
            app,
            &mut InitFrameResources {
                render_pass: Some(resources.render_pass.as_ref().unwrap()),
                upload_cmd: Some(&resources.vec_cmd_primary[0]),
                fence: Some(&resources.vec_fence[0]),
            }
        )?)
    );

    // 2. Framebuffers (по одному на image)]
    resources.init_framebuffer(app)?;

    resources.start_time = std::time::Instant::now();

    Ok(())

}
