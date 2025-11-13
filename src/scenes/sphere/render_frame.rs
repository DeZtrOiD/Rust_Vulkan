use ash::vk;
use super::frame_resources::FrameResources;
use super::super::super::vulkan_wr::{
    app::VulkanApp,
};

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
