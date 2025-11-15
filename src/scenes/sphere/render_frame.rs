use ash::vk;
use super::frame_resources::FrameResources;
use super::super::super::vulkan_wr::{
    app::VulkanApp,
    ImGui_wr::ImGUIUniform
};
use imgui::internal::RawWrapper;

pub fn render_frame_app(app: & mut VulkanApp, resources: &mut FrameResources) -> Result<(), &'static str> {
    // fecne связывает CPU GPU, индексируется фреймом, image_available принадлежит также фрейму.
    // Тк когда queue_submit закончит работу, этот фрейм снова освободится через fence
    // cmd buf и render_finished индексируются картинкой, тк тесно с ней связаны.
    let current_frame: usize = app.frame_index;
    
    let frame_sync = resources.vec_fence[current_frame].fence;
    unsafe {
        app.core._logical_device.wait_for_fences(&[frame_sync], true, u64::MAX).unwrap();
        app.core._logical_device.reset_fences(&[frame_sync]).unwrap();
    }
    
    let sem_offset = (current_frame * 2) as usize;
    let image_available = &resources.vec_sem[sem_offset];
    
    let (image_index, _) = app.swapchain.acquire_next_image(Some(image_available.semaphore), None)?;

    // let fence_2 = resources.vec_fence[app.swapchain.images.len() + image_index as usize].fence;
    // unsafe {
    //     app.core._logical_device.wait_for_fences(&[fence_2], true, u64::MAX).unwrap();
    //     app.core._logical_device.reset_fences(&[fence_2]).unwrap();
    // }


    let cmd_secondary_imgui = &resources.vec_cmd_secondary_imgui[current_frame];
    // Обновление ImGui и перезапись его secondary командного буфера
    if let Some(imgui) = resources._imgui.as_mut() {
        imgui.render_frame(
            current_frame as u32,
            cmd_secondary_imgui,
            resources.render_pass.as_ref().unwrap().render_pass,
            resources.framebuffers[image_index as usize].framebuffer,
            &mut app.window
        )?;
    }

    let cmd_primary = &resources.vec_cmd_primary[current_frame as usize];

    // Основной буфер команд, который включает в себя secondary
    {
        cmd_primary.reset(None)?;
        cmd_primary.begin(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE, None)?;
        let clear_values = [
            vk::ClearValue { color: vk::ClearColorValue { float32: [0.0, 0.51, 0.12, 1.0] } },
            vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 } }
        ];
        let begin_info = vk::RenderPassBeginInfo {
            render_pass: resources.render_pass.as_ref().unwrap().render_pass,
            framebuffer: resources.framebuffers[image_index as usize].framebuffer,
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: app.swapchain.extent
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
            ..Default::default()
        };
        unsafe {
            cmd_primary._device.cmd_begin_render_pass(
                cmd_primary._buffer,
                &begin_info,
                vk::SubpassContents::SECONDARY_COMMAND_BUFFERS  // specifying how the commands in the first subpass will be provided.
            );

            // основной цикл
            cmd_primary._device.cmd_execute_commands(
                cmd_primary._buffer,
                &[
                    resources.vec_cmd_secondary[image_index as usize]._buffer,
                    resources.vec_cmd_secondary_imgui[current_frame as usize]._buffer
                ]
            );
            cmd_primary._device.cmd_end_render_pass(cmd_primary._buffer);
        }
        cmd_primary.end()?;
    }


    // let cmd_primary = &resources.vec_cmd_primary[image_index as usize];
    let render_finished = &resources.vec_sem[(image_index * 2 + 1) as usize];

    // Submit
    let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

    let submit_info = vk::SubmitInfo {
        wait_semaphore_count: 1,
        p_wait_semaphores: &image_available.semaphore,
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

    app.swapchain.queue_present(app.core._graphics_queue, &present_info)?;

    Ok(())

}



/*
use ash::vk;
use super::frame_resources::FrameResources;
use super::super::super::vulkan_wr::{
    app::VulkanApp,
    ImGui_wr::ImGUIUniform
};
use imgui::internal::RawWrapper;

pub fn render_frame_app(app: & mut VulkanApp, resources: &mut FrameResources) -> Result<(), &'static str> {
    // fecne связывает CPU GPU, индексируется фреймом, image_available принадлежит также фрейму.
    // Тк когда queue_submit закончит работу, этот фрейм снова освободится через fence
    // cmd buf и render_finished индексируются картинкой, тк тесно с ней связаны.
    let current_frame = app.frame_index;
    
    let frame_sync = resources.vec_fence[current_frame].fence;
    unsafe {
        app.core._logical_device.wait_for_fences(&[frame_sync], true, u64::MAX).unwrap();
        app.core._logical_device.reset_fences(&[frame_sync]).unwrap();
    }
    
    let sem_offset = (current_frame * 2) as usize;
    let image_available = &resources.vec_sem[sem_offset];
    
    let (image_index, _) = app.swapchain.acquire_next_image(Some(image_available.semaphore), None)?;

    // let fence_2 = resources.vec_fence[app.swapchain.images.len() + image_index as usize].fence;
    // unsafe {
    //     app.core._logical_device.wait_for_fences(&[fence_2], true, u64::MAX).unwrap();
    //     app.core._logical_device.reset_fences(&[fence_2]).unwrap();
    // }


    let cmd_secondary_imgui = &resources.vec_cmd_secondary_imgui[image_index as usize];
    // Обновление ImGui и перезапись его secondary командного буфера
    if let Some(imgui) = resources._imgui.as_mut() {
        imgui.render_frame(
            image_index,
            cmd_secondary_imgui,
            resources.render_pass.as_ref().unwrap().render_pass,
            resources.framebuffers[image_index as usize].framebuffer,
            &mut app.window
        )?;
    }

    let cmd_primary = &resources.vec_cmd_primary[image_index as usize];

    // Основной буфер команд, который включает в себя secondary
    {
        cmd_primary.reset(None)?;
        cmd_primary.begin(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE, None)?;
        let clear_values = [
            vk::ClearValue { color: vk::ClearColorValue { float32: [0.0, 0.51, 0.12, 1.0] } },
            vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 } }
        ];
        let begin_info = vk::RenderPassBeginInfo {
            render_pass: resources.render_pass.as_ref().unwrap().render_pass,
            framebuffer: resources.framebuffers[image_index as usize].framebuffer,
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: app.swapchain.extent
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
            ..Default::default()
        };
        unsafe {
            cmd_primary._device.cmd_begin_render_pass(
                cmd_primary._buffer,
                &begin_info,
                vk::SubpassContents::SECONDARY_COMMAND_BUFFERS  // specifying how the commands in the first subpass will be provided.
            );

            // основной цикл
            cmd_primary._device.cmd_execute_commands(
                cmd_primary._buffer,
                &[
                    resources.vec_cmd_secondary[image_index as usize]._buffer,
                    resources.vec_cmd_secondary_imgui[image_index as usize]._buffer
                ]
            );
            cmd_primary._device.cmd_end_render_pass(cmd_primary._buffer);
        }
        cmd_primary.end()?;
    }


    // let cmd_primary = &resources.vec_cmd_primary[image_index as usize];
    let render_finished = &resources.vec_sem[(image_index * 2 + 1) as usize];

    // Submit
    let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

    let submit_info = vk::SubmitInfo {
        wait_semaphore_count: 1,
        p_wait_semaphores: &image_available.semaphore,
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

    app.swapchain.queue_present(app.core._graphics_queue, &present_info)?;

    Ok(())

}

*/