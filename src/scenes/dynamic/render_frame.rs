use ash::vk;
use crate::scenes::dynamic::renderable_object::RenderObjectEnum;
use crate::vulkan_wr::app::SceneResources;
use crate::vulkan_wr::renderable_traits::RenderFrameResources;

use super::frame_resources::FrameResources;
use super::super::super::vulkan_wr::{
    app::VulkanApp,
    renderable_traits::{RenderObject},
    ImGui_wr::ImguiResources,
};


pub fn render_frame_app<R: ImguiResources + Default>(app: & mut VulkanApp, resources: &mut FrameResources<R>) -> Result<(), &'static str> {
    // fecne связывает CPU GPU, индексируется фреймом, image_available принадлежит также фрейму.
    // Тк когда queue_submit закончит работу, этот фрейм снова освободится через fence
    // cmd buf и render_finished индексируются картинкой, тк тесно с ней связаны.
    if (0, 0) == app.window.get_width_height() {
        app.frame_index = (app.frame_index + 1) % app.image_count;
        return Ok(());
    }
    let swap_extent = app.swapchain.extent;
    if swap_extent.height <= 1 || swap_extent.width <= 1 {
        app.frame_index = (app.frame_index + 1) % app.image_count;
        return Ok(());
    }
    let current_frame: usize = app.frame_index as usize;
    
    let frame_sync = resources.vec_fence[current_frame].fence;
    unsafe {
        app.core._logical_device.wait_for_fences(&[frame_sync], true, u64::MAX).unwrap();
        app.core._logical_device.reset_fences(&[frame_sync]).unwrap();
    }
    
    let sem_offset = (current_frame * 2) as usize;
    let image_available = resources.vec_sem[sem_offset].semaphore.clone();
    let shadow_finished = &resources.shadow_finished_sem[current_frame].semaphore;

    let (image_index, suboptimal) = app.swapchain.acquire_next_image(Some(image_available), None)?;
    if suboptimal {
            // Swapchain полностью устарел, нужно пересоздать
            app.device_wait_idle()?;
            app.recreate_swapchain()?;
            resources.init_framebuffer(&app)?;
            app.frame_index = (app.frame_index + 1) % app.image_count;
            app.device_wait_idle()?;
            return Ok(());
    }

    // Рендерим shadow pass перед основным рендерингом
    for obj in &mut resources.vec_objects {
        if let RenderObjectEnum::Shadows(shadow_obj) = obj {
            shadow_obj.render_shadow_pass(app)?;
        }
    }
    // Получаем командные буферы для теней
    let mut shadow_command_buffers = Vec::new();
    for obj in &resources.vec_objects {
        if let RenderObjectEnum::Shadows(shadow_obj) = obj {
            shadow_command_buffers.push(shadow_obj.shadow_cmd_vec[current_frame as usize]._buffer);
        }
    }
    
    let resss = RenderFrameResources{
            // render_pass: Some(resources.render_pass.as_ref().unwrap()),
            // framebuffer: Some(&resources.framebuffers[image_index as usize]),
            ..Default::default()
    };
    for obj in &mut resources.vec_objects {
        obj.render(app, &resss)?;
    }
    let cmd_primary = &resources.vec_cmd_primary[current_frame as usize];
    
    // Основной буфер команд, который включает в себя secondary
    {
        cmd_primary.reset(None)?;
        cmd_primary.begin(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE, None)?;

        let color_attachment = vk::RenderingAttachmentInfo {
            image_view: resources.image_view[image_index as usize].view,
            image_layout: vk::ImageLayout::ATTACHMENT_OPTIMAL,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            clear_value: vk::ClearValue { 
                color: vk::ClearColorValue { float32: [10.0/255.0, 10.0/255.0, 50.0/255.0, 1.0] } 
            },
            ..Default::default()
        };
        
        let depth_attachment = vk::RenderingAttachmentInfo {
            image_view: resources.depth_image_views[image_index as usize].view,
            image_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::DONT_CARE,
            clear_value: vk::ClearValue { 
                depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 } 
            },
            ..Default::default()
        };
        
        let rendering_info = vk::RenderingInfo {
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swap_extent,
            },
            layer_count: 1,
            color_attachment_count: 1,
            p_color_attachments: &color_attachment,
            p_depth_attachment: &depth_attachment,
            flags: vk::RenderingFlags::CONTENTS_SECONDARY_COMMAND_BUFFERS,
            ..Default::default()
        };

        unsafe {

            let color_image_barrier = vk::ImageMemoryBarrier {
                src_access_mask: vk::AccessFlags::empty(), // нет предыдущих операций
                dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                old_layout: vk::ImageLayout::UNDEFINED,
                new_layout: vk::ImageLayout::ATTACHMENT_OPTIMAL,
                image: app.swapchain.images[image_index as usize],
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                ..Default::default()
            };
            let depth_image_barrier = vk::ImageMemoryBarrier {
                src_access_mask: vk::AccessFlags::empty(),
                dst_access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                old_layout: vk::ImageLayout::UNDEFINED,
                new_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                image: resources.depth_images[image_index as usize].image,
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                ..Default::default()
            };

            cmd_primary.pipeline_barrier(
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,  // dst_stage_mask
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[color_image_barrier, depth_image_barrier],
            );


            cmd_primary.begin_dynamic_rendering(&rendering_info)?;


            let buff_vec: Vec<vk::CommandBuffer> = resources.vec_objects.iter().map(|obj|
                match obj {
                    RenderObjectEnum::ImGui(objj) => {objj.cmd_vec[current_frame as usize]._buffer}
                    #[cfg(feature = "scene1")]
                    RenderObjectEnum::Sphere(objj) => {objj.cmd_vec[current_frame as usize]._buffer}
                    #[cfg(feature = "scene2")]
                    RenderObjectEnum::Light(objj) => {objj.cmd_vec[current_frame as usize]._buffer}
                    #[cfg(feature = "scene3")]
                    RenderObjectEnum::Shadows(objj) => {objj.cmd_vec[current_frame as usize]._buffer}
                }
            ).collect();

            // основной цикл
            cmd_primary.execute_commands(
                buff_vec.as_slice()
            );
            cmd_primary.end_dynamic_rendering()?;

            let image_memory_barrier = vk::ImageMemoryBarrier {
                src_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                dst_access_mask: vk::AccessFlags::MEMORY_READ,
                old_layout: vk::ImageLayout::ATTACHMENT_OPTIMAL,
                new_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                image: app.swapchain.images[image_index as usize],
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                ..Default::default()
            };
            cmd_primary.pipeline_barrier(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_memory_barrier],
            );


        }
        cmd_primary.end()?;
    }

    let render_finished = &(&resources.vec_sem[(image_index * 2 + 1) as usize]).semaphore;

    // Submit
    let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];


    let shadow_submit = vk::SubmitInfo {
        wait_semaphore_count: 1,
        p_wait_semaphores: &image_available,
        p_wait_dst_stage_mask: &vk::PipelineStageFlags::TOP_OF_PIPE,
        command_buffer_count: shadow_command_buffers.len() as u32,
        p_command_buffers: shadow_command_buffers.as_ptr(),
        signal_semaphore_count: 1,
        p_signal_semaphores: shadow_finished,
        ..Default::default()
    };


    let submit_info = vk::SubmitInfo {
        wait_semaphore_count: 1,
        p_wait_semaphores: shadow_finished,
        p_wait_dst_stage_mask: wait_stages.as_ptr(),
        command_buffer_count: 1,
        p_command_buffers: &cmd_primary._buffer,
        signal_semaphore_count: 1,
        p_signal_semaphores: render_finished,
        ..Default::default()
    };

    app.core.queue_submit(&[shadow_submit, submit_info], frame_sync)?;

    // Present
    let present_info = vk::PresentInfoKHR {
        wait_semaphore_count: 1,
        p_wait_semaphores: render_finished,
        swapchain_count: 1,
        p_swapchains: &app.swapchain.swapchain,
        p_image_indices: &image_index,
        ..Default::default()
    };

    // app.swapchain.queue_present(app.core._graphics_queue, &present_info)?;
    match app.swapchain.queue_present(app.core._graphics_queue, &present_info) {
        Ok(true) => {
            // Все в порядке
        },
        Ok(false) => {
            // Swapchain полностью устарел, нужно пересоздать
            app.device_wait_idle()?;
            app.recreate_swapchain()?;
            resources.init_framebuffer(&app)?;
            app.frame_index = (app.frame_index + 1) % app.image_count;
            app.device_wait_idle()?;
        },
        Err(e) => {
            return Err("Failed to present swapchain image");
        }
    };

    Ok(())

}
