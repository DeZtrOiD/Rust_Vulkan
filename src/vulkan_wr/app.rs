// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: основное апп
// TODO: РАСТ ДАЕТ ВОЗМОЖНОСТЬ УСТАНОВИТЬ ПОРЯДОК ДРОПА. ДУМАЙТЕ
// TODO: нужно слить его с window
// TODO: надо что-то сделать с преедачей параметров везде
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use super::core::{VulkanCore, VulkanCoreBuilder};
use super::swapchain::{VulkanSwapchain, VulkanSwapchainBuilder};
use crate::vulkan_wr::command_pb::command_buffer::VulkanCommandBuffer;
use crate::window::Window;
use super::command_pb::command_pool::VulkanCommandPool;
use ash::{vk, khr};
use super::descriptor::descriptor_pool::VulaknDescriptorPool;
use super::semaphore::VulkanSemaphore;
use super::fence::VulkanFence;


pub type AppVkResult<T> = Result<T, &'static str>;

pub struct FrameResources {
    pub vec_sem: Vec<VulkanSemaphore>, // [image_available, render_finished, ...]
    pub vec_cmd: Vec<VulkanCommandBuffer>,
    pub vec_fence: Vec<VulkanFence>,  // CPU + GPU
}

pub struct VulkanApp {
    pub descriptor_pool: VulaknDescriptorPool,
    pub command_pool: VulkanCommandPool,
    pub swapchain: VulkanSwapchain,
    pub core: VulkanCore,
    pub frame_index: usize,
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
    pub fn init(&mut self, init: fn(app: &mut VulkanApp) -> AppVkResult<()>) -> AppVkResult<()> {
        init(self)
    }

    pub fn update(&mut self, update: fn(app: &mut VulkanApp) -> AppVkResult<()>) -> AppVkResult<()> {
        update(self)
    }


    pub fn record_render_graph(
            &mut self,
            render: fn(
                app: &mut VulkanApp,
                resources: &FrameResources
            ) -> AppVkResult<()>,
            resources: &FrameResources
        ) -> AppVkResult<()> {
        
        render(self, resources)
    }

    pub fn submit_and_present(&mut self,
        present: fn(
            app: &mut VulkanApp,
            resources: &FrameResources
        ) -> AppVkResult<()>,
        resources: &FrameResources
    ) -> Result<(), &'static str> {
        self.frame_index = (self.frame_index + 1) % self.swapchain.images.len();
        present(self, resources)
    }

    pub fn shutdown(&mut self, shutdown: fn(app: &mut VulkanApp) -> AppVkResult<()>) -> AppVkResult<()> {
        shutdown(self)
    }

    pub fn get_frame_resources(
            &self,
            cmd_count: u32,
            cmd_level: vk::CommandBufferLevel,
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

        let vec_cmd = self.command_pool.allocate_command_buffers(cmd_count, cmd_level)?;
        Ok(FrameResources {
            vec_sem: vec_sem,
            vec_cmd: vec_cmd,
            vec_fence: vec_fence,
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

