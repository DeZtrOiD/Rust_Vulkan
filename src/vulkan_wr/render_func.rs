// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: 
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#

use super::app::{VulkanApp, FrameResources};
use ash::vk;


pub fn init_app(app: &mut VulkanApp) -> Result<(), &'static str> {
    Ok(())
}

pub fn update_app(app: &mut VulkanApp) -> Result<(), &'static str> {
    Ok(())
}

pub fn record_render_commands_app(app: & mut VulkanApp, resources: &FrameResources) -> Result<(), &'static str> {
    // vk::ClearColorValue - чистит цвет (°〇°), то что ниже юнион, в который можно засунуть чистку глубины
    let clear_value = vk::ClearValue {
        color: vk::ClearColorValue {
            float32: [164.0 / 256.0, 30.0 / 256.0, 34.0 / 256.0, 1.0],
        },
    };
    let ranges = vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
    };

    for (i, cmd) in resources.vec_cmd.iter().enumerate() {
        cmd.begin(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE, None)?;

        let barrier = vk::ImageMemoryBarrier {
            old_layout: vk::ImageLayout::UNDEFINED,
            new_layout: vk::ImageLayout::GENERAL,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image: app.swapchain.images[i],
            subresource_range: ranges,
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
            ..Default::default()
        };

        unsafe {
            cmd._device.cmd_pipeline_barrier(
                cmd._buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );

            cmd._device.cmd_clear_color_image(
                cmd._buffer,
                app.swapchain.images[i],
                vk::ImageLayout::GENERAL,
                &clear_value.color,
                &[ranges],
            );
            let barrier_to_present = vk::ImageMemoryBarrier {
                old_layout: vk::ImageLayout::GENERAL,
                new_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                image: app.swapchain.images[i],
                subresource_range: ranges,
                src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                dst_access_mask: vk::AccessFlags::empty(),
                ..Default::default()
            };

            cmd._device.cmd_pipeline_barrier(
                cmd._buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier_to_present],
            );
        }

        cmd.end()?;
    }
    Ok(())
}

pub fn present_frame_app(app: & mut VulkanApp, resources: &FrameResources) -> Result<(), &'static str> {
    // fecne связывает CPU GPU, индексируется фреймом, image_available принадлежит так же фрейму.
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
    let cmd_buffer = &resources.vec_cmd[image_index as usize];
    let render_finished = &resources.vec_sem[(image_index * 2 + 1) as usize];

    // Submit
    let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

    let submit_info = vk::SubmitInfo {
        wait_semaphore_count: 1,
        p_wait_semaphores: &image_available.semaphore,
        p_wait_dst_stage_mask: wait_stages.as_ptr(),
        command_buffer_count: 1,
        p_command_buffers: &cmd_buffer._buffer,
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

pub fn shutdown_app(app: & mut VulkanApp) -> Result<(), &'static str> {
    Ok(())
}
