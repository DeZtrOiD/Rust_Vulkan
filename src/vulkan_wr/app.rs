// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: основное апп
// TODO: РАСТ ДАЕТ ВОЗМОЖНОСТЬ УСТАНОВИТЬ ПОРЯДОК ДРОПА. ДУМАЙТЕ
// TODO: нужно слить его с window
// TODO: надо что-то сделать с преедачей параметров везде
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#

use ash::{vk, khr};
use super::core::{VulkanCore, VulkanCoreBuilder};
use super::swapchain::{VulkanSwapchain, VulkanSwapchainBuilder};
use crate::vulkan_wr::command_pb::command_buffer::VulkanCommandBuffer;
use crate::vulkan_wr::image::image_view::VulkanImageView;
use crate::window::Window;
use super::command_pb::command_pool::VulkanCommandPool;
use super::descriptor::{
    descriptor_pool::VulaknDescriptorPool,
    descriptor_set_layout::VulkanDescriptorSetLayout,
    descriptor_set::VulkanDescriptorSet
};

use super::semaphore::VulkanSemaphore;
use super::fence::VulkanFence;
use super::framebuffer::VulkanFramebuffer;
use super::render_pass::pass::VulkanRenderPass;
use super::pipeline::{
    pipeline::VulkanPipeline,
    pipeline_layout::VulkanPipelineLayout
};
use super::buffer::buffer::VulkanBuffer;

pub type AppVkResult<T> = Result<T, &'static str>;

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

    pub depth_images: Vec<super::image::image::VulkanImage>,
    pub depth_image_views: Vec<VulkanImageView>,

    pub start_time: std::time::Instant,
}

pub struct VulkanApp {
    pub frame_index: usize,
    pub command_pool: VulkanCommandPool,
    pub descriptor_pool: VulaknDescriptorPool,
    pub swapchain: VulkanSwapchain,
    pub core: VulkanCore,
}

impl VulkanApp {
    pub fn try_new(window: &Window, app_name: &str) -> AppVkResult<Self> {
        let vk_core = VulkanCoreBuilder::new(app_name)
            .api_version(1, 2, 0)
            .enable_validation(cfg!(debug_assertions))
            .build(window)?;
        let vk_swapchain = VulkanSwapchainBuilder::new(&vk_core)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
            .build()?;


        let cmd_pool = VulkanCommandPool::try_new(
            &vk_core._logical_device,
            vk_core._graphics_queue_index,
            vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER
        )?;
        
        let pool_size = vec![vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 4 as u32
        }];
        let dsc_pool = VulaknDescriptorPool::try_new(&vk_core._logical_device, &pool_size,  5 as u32, None)?;
        Ok( Self {
            core: vk_core,
            swapchain: vk_swapchain,
            command_pool: cmd_pool,
            descriptor_pool: dsc_pool,
            frame_index: 0,
        })
    }

    // Создать:
    // - render pass
    // - pipeline
    // - descriptor sets/layouts
    // - framebuffers (по одному на swapchain image)
    // - vertex/uniform buffers
    // - загрузить текстуры (через staging + fence)
    pub fn init(
        &mut self,
        init: fn(app: &mut VulkanApp, resources: &mut FrameResources) -> AppVkResult<()>,
        resources: & mut FrameResources
    ) -> AppVkResult<()> {
        init(self, resources)
    }

    pub fn update(
            &mut self,
            update: fn(app: &mut VulkanApp, resources: &mut FrameResources) -> AppVkResult<()>,
            resources: &mut FrameResources
        ) -> AppVkResult<()> {
        update(self, resources)
    }

    pub fn render(&mut self,
        present: fn(
            app: &mut VulkanApp,
            resources: &mut FrameResources
        ) -> AppVkResult<()>,
        resources: &mut FrameResources
    ) -> Result<(), &'static str> {
        self.frame_index = (self.frame_index + 1) % self.swapchain.images.len();
        present(self, resources)
    }

    pub fn shutdown(&mut self, shutdown: fn(
            app: &mut VulkanApp,
            resources: &mut FrameResources
        ) -> AppVkResult<()>,
        resources: &mut FrameResources
    ) -> AppVkResult<()> {
        shutdown(self, resources)
    }

    pub fn get_frame_resources(
            &self,
            cmd_count_primary: u32,
            cmd_count_secondary: u32,
            sem_count: u32,
            fence_count: u32
        ) -> AppVkResult<FrameResources>{
        let mut vec_sem = vec![];
        for _ in 0..sem_count {
            vec_sem.push(VulkanSemaphore::try_new(&self.core._logical_device)?);
        }

        let mut vec_fence = vec![];
        for _ in 0..fence_count {
            vec_fence.push(VulkanFence::try_new(&self.core._logical_device, vk::FenceCreateFlags::SIGNALED)?);
        }

        let mut vec_cmd = vec![];
        if cmd_count_primary != 0 {
            vec_cmd = self.command_pool.allocate_command_buffers(cmd_count_primary, vk::CommandBufferLevel::PRIMARY)?;
        }
        if cmd_count_secondary != 0 {
            let mut tmp = self.command_pool.allocate_command_buffers(cmd_count_primary, vk::CommandBufferLevel::PRIMARY)?;
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

    pub fn get_swapchain_images_count(&self) -> u32 {
        self.swapchain.images.len() as u32
    }

    pub fn device_wait_idle(&self) -> Result<(), &'static str>{
        unsafe {
            self.core._logical_device.device_wait_idle().map_err(|_| "Err device_wait_idle")
        }
    }

}

