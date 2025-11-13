
use ash::vk;
use super::super::super::vulkan_wr::{
    app::{VulkanApp, SceneResources},
    render_pass::pass::VulkanRenderPass,
    descriptor::{descriptor_set_layout::VulkanDescriptorSetLayout, descriptor_set::VulkanDescriptorSet},
    pipeline::{pipeline_layout::VulkanPipelineLayout, pipeline::VulkanPipeline},
    buffer::buffer::VulkanBuffer,
    framebuffer::VulkanFramebuffer,
    image::{image_view::VulkanImageView, image::VulkanImage},
    command_pb::command_buffer::VulkanCommandBuffer,
    sync::{
        semaphore::VulkanSemaphore,
        fence::VulkanFence,
    }
};

pub struct FrameResources {
    pub vec_fence: Vec<VulkanFence>,  // CPU + GPU
    pub vec_sem: Vec<VulkanSemaphore>, // [image_available, render_finished, ...]
    pub vec_cmd: Vec<VulkanCommandBuffer>,

    pub image_view: Vec<VulkanImageView>,
    pub framebuffers: Vec<VulkanFramebuffer>, // one per swapchain image

    pub render_pass: Option<VulkanRenderPass>,

    pub descriptor_sets: Vec<VulkanDescriptorSet>,
    pub pipeline: Option<VulkanPipeline>,
    pub descriptor_set_layout: Vec<VulkanDescriptorSetLayout>,
    pub pipeline_layout: Vec<VulkanPipelineLayout>,

    pub uniform_buffers: Vec<VulkanBuffer>, // per-image UBO (float time)
    pub vertex_buffer: Option<VulkanBuffer>,
    pub index_buffer: Option<VulkanBuffer>,
    pub index_count: u32,

    pub depth_images: Vec<VulkanImage>,
    pub depth_image_views: Vec<VulkanImageView>,

    pub start_time: std::time::Instant,
}

impl SceneResources for FrameResources {}

pub fn get_frame_resources(
        app: &VulkanApp,
        cmd_count_primary: u32,
        cmd_count_secondary: u32,
        sem_count: u32,
        fence_count: u32
    ) -> Result<FrameResources, &'static str>{
    let mut vec_sem = vec![];
    for _ in 0..sem_count {
        vec_sem.push(VulkanSemaphore::try_new(&app.core._logical_device)?);
    }

    let mut vec_fence = vec![];
    for _ in 0..fence_count {
        vec_fence.push(VulkanFence::try_new(&app.core._logical_device, vk::FenceCreateFlags::SIGNALED)?);
    }

    let mut vec_cmd = vec![];
    if cmd_count_primary != 0 {
        vec_cmd = app.command_pool.allocate_command_buffers(cmd_count_primary, vk::CommandBufferLevel::PRIMARY)?;
    }
    if cmd_count_secondary != 0 {
        let mut tmp = app.command_pool.allocate_command_buffers(cmd_count_primary, vk::CommandBufferLevel::PRIMARY)?;
        vec_cmd.append(&mut tmp);
    }
    Ok(FrameResources {
        vec_sem: vec_sem,
        vec_cmd: vec_cmd,
        vec_fence: vec_fence,
        image_view: vec![],
        framebuffers: vec![], // one per swapchain image
        render_pass: None,
        pipeline: None,
        pipeline_layout: vec![],
        descriptor_set_layout: vec![],
        descriptor_sets: vec![],
        uniform_buffers: vec![], // per-image UBO (float time)
        vertex_buffer: None,
        index_buffer: None,
        index_count: 0,
        depth_image_views: vec![],
        depth_images: vec![],
        start_time: std::time::Instant::now()
    })
}
