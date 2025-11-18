use ash::vk;
use crate::scenes::common::renderable_object::RenderObjectEnum;
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
    let swap_exten = app.swapchain.extent;
    if swap_exten.height <= 1 || swap_exten.width <= 1 {
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

    let resss = RenderFrameResources{
            render_pass: Some(resources.render_pass.as_ref().unwrap()),
            framebuffer: Some(&resources.framebuffers[image_index as usize]),
    };
    for obj in &mut resources.vec_objects {
        obj.render(app, &resss)?;
    }

    let cmd_primary = &resources.vec_cmd_primary[current_frame as usize];

    // Основной буфер команд, который включает в себя secondary
    {
        cmd_primary.reset(None)?;
        cmd_primary.begin(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE, None)?;
        let clear_values = [
            vk::ClearValue { color: vk::ClearColorValue { float32: [92.0/255.0, 234.0/255.0, 197.0/255.0, 1.0] } },
            vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 } }
        ];
        let begin_info = vk::RenderPassBeginInfo {
            render_pass: resources.render_pass.as_ref().unwrap().render_pass,
            framebuffer: resources.framebuffers[image_index as usize].framebuffer,
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swap_exten
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
            ..Default::default()
        };
        unsafe {
            cmd_primary.begin_render_pass(
                &begin_info,
                vk::SubpassContents::SECONDARY_COMMAND_BUFFERS  // specifying how the commands in the first subpass will be provided.
            );

            let buff_vec: Vec<vk::CommandBuffer> = resources.vec_objects.iter().map(|obj|
                match obj {
                    RenderObjectEnum::ImGui(objj) => {objj.cmd_vec[current_frame as usize]._buffer}
                    RenderObjectEnum::Sphere(objj) => {objj.cmd_vec[current_frame as usize]._buffer}
                }
            ).collect();

            // основной цикл
            cmd_primary.execute_commands(
                buff_vec.as_slice()
            );
            cmd_primary.end_render_pass();
        }
        cmd_primary.end()?;
    }

    let render_finished = &resources.vec_sem[(image_index * 2 + 1) as usize];

    // Submit
    let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

    let submit_info = vk::SubmitInfo {
        wait_semaphore_count: 1,
        p_wait_semaphores: &image_available,
        p_wait_dst_stage_mask: wait_stages.as_ptr(),
        command_buffer_count: 1,
        p_command_buffers: &cmd_primary._buffer,
        signal_semaphore_count: 1,
        p_signal_semaphores: &render_finished.semaphore,
        ..Default::default()
    };

    app.core.queue_submit(&[submit_info], frame_sync)?;

    // Present
    let present_info = vk::PresentInfoKHR {
        wait_semaphore_count: 1,
        p_wait_semaphores: &render_finished.semaphore,
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
