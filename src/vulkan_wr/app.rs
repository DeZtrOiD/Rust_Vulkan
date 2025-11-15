// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: основное апп
// TODO: РАСТ ДАЕТ ВОЗМОЖНОСТЬ УСТАНОВИТЬ ПОРЯДОК ДРОПА. ДУМАЙТЕ
// TODO: нужно слить его с window
// TODO: надо что-то сделать с преедачей параметров везде
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#

use ash::{vk};
use super::core::{VulkanCore, VulkanCoreBuilder};
use super::swapchain::{VulkanSwapchain, VulkanSwapchainBuilder};
use crate::window::Window;
use super::command_pb::command_pool::VulkanCommandPool;
use super::descriptor::descriptor_pool::VulaknDescriptorPool;

pub type AppVkResult<T> = Result<T, &'static str>;

// Трейт для использования шаблонов в типах ресурсов
pub trait SceneResources {}

pub struct VulkanApp {
    pub frame_index: usize,
    pub command_pool: VulkanCommandPool,
    pub descriptor_pool: VulaknDescriptorPool,
    pub swapchain: VulkanSwapchain,
    pub core: VulkanCore,
    pub window: Window,
}

impl VulkanApp {
    pub fn try_new(window: Window, app_name: &str) -> AppVkResult<Self> {
        let vk_core = VulkanCoreBuilder::new(app_name)
            .api_version(1, 2, 0)
            .enable_validation(cfg!(debug_assertions))
            .build(&window)?;
        let vk_swapchain = VulkanSwapchainBuilder::new(&vk_core)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
            .build()?;


        let cmd_pool = VulkanCommandPool::try_new(
            &vk_core._logical_device,
            vk_core._graphics_queue_index,
            vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER
        )?;

        let pool_size = vec![
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: vk_swapchain.images.len() as u32 * 2
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: vk_swapchain.images.len() as u32
            },
        ];
        let max_sets = vk_swapchain.images.len() as u32 * 3;
        let dsc_pool = VulaknDescriptorPool::try_new(
            &vk_core._logical_device,
            &pool_size, 
            max_sets,
            None
        )?;
        Ok( Self {
            core: vk_core,
            swapchain: vk_swapchain,
            command_pool: cmd_pool,
            descriptor_pool: dsc_pool,
            frame_index: 0,
            window: window,
        })
    }

    // Создать:
    // - render pass
    // - pipeline
    // - descriptor sets/layouts
    // - framebuffers (по одному на swapchain image)
    // - vertex/uniform buffers
    // - загрузить текстуры (через staging + fence)
    pub fn init<R: SceneResources>(
        &mut self,
        init: fn(app: &mut VulkanApp, resources: &mut R) -> AppVkResult<()>,
        resources: & mut R
    ) -> AppVkResult<()> {
        init(self, resources)
    }

    pub fn update<R: SceneResources>(
            &mut self,
            update: fn(app: &mut VulkanApp, resources: &mut R) -> AppVkResult<()>,
            resources: &mut R
        ) -> AppVkResult<()> {
        self.window.process_events();
        update(self, resources)
    }

    pub fn render<R: SceneResources>(&mut self,
        present: fn(
            app: &mut VulkanApp,
            resources: &mut R
        ) -> AppVkResult<()>,
        resources: &mut R
    ) -> Result<(), &'static str> {
        self.frame_index = (self.frame_index + 1) % self.swapchain.images.len();
        present(self, resources)
    }

    pub fn shutdown<R: SceneResources>(&mut self, shutdown: fn(
            app: &mut VulkanApp,
            resources: &mut R
        ) -> AppVkResult<()>,
        resources: &mut R
    ) -> AppVkResult<()> {
        shutdown(self, resources)
    }

    pub fn get_frame_resources<R: SceneResources>(
            &self,
            image_count: u32,
            func: fn(
                app: &VulkanApp,
                image_count: u32
            ) -> AppVkResult<R>
        ) -> AppVkResult<R>{
            func(
                &self,
                image_count
            )
    }

    pub fn get_swapchain_images_count(&self) -> u32 {
        self.swapchain.images.len() as u32
    }

    pub fn device_wait_idle(&self) -> Result<(), &'static str>{
        unsafe {
            self.core._logical_device.device_wait_idle().map_err(|_| "Err device_wait_idle")
        }
    }

    pub fn should_close(&self) -> bool {
        self.window.should_close()
    }

}

