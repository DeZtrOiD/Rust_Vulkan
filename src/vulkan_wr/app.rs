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
use crate::window::Window;
use super::command_pb::command_pool::VulkanCommandPool;
use ash::{vk, khr};
use ash::vk::Fence;
use super::descriptor::descriptor_pool::VulaknDescriptorPool;


pub type AppVkResult<T> = Result<T, &'static str>;

pub struct VulkanApp {
    _descriptor_pool: VulaknDescriptorPool,
    _command_pool: VulkanCommandPool,
    _swapchain: VulkanSwapchain,
    _core: VulkanCore,
}

impl VulkanApp {
    pub fn try_new(window: &Window, app_name: &str) -> AppVkResult<Self> {
        let vk_core = VulkanCoreBuilder::new(app_name)
            .api_version(1, 3, 0)
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
            _core: vk_core,
            _swapchain: vk_swapchain,
            _command_pool: cmd_pool,
            _descriptor_pool: dsc_pool,
        })
    }

    pub fn run(&self) -> AppVkResult<()> {
        let cmb = self._command_pool.allocate_command_buffers(
                self._swapchain.images.len() as u32,
                vk::CommandBufferLevel::PRIMARY
        )?;
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let image_available = unsafe {
            self._core._logical_device
                .create_semaphore(&semaphore_info, None)
                .map_err(|_| "Failed to create semaphore")?
        };
        let render_finished = unsafe {
            self._core._logical_device.create_semaphore(&semaphore_info, None).unwrap()
        };


        // vk::ClearColorValue - чистит цвет (°〇°), то что ниже юнион, в который можно засунуть чистку глубины
        let clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [164.0/256.0, 30.0/256.0, 34.0/256.0, 0.0]
            }
        };
        let ranges = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            base_array_layer: 0,
            level_count: 1,
            layer_count: 1,
        };

        for i in 0..cmb.len() {
            cmb[i].begin(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE, None)?;

            let barrier = vk::ImageMemoryBarrier {
                old_layout: vk::ImageLayout::UNDEFINED,
                new_layout: vk::ImageLayout::GENERAL,
                src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                image: self._swapchain.images[i],
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                src_access_mask: vk::AccessFlags::empty(),
                dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                ..Default::default()
            };

            unsafe {
                cmb[i]._device.cmd_pipeline_barrier(
                    cmb[i]._buffer,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[barrier],
                );
            }

            unsafe { cmb[i]._device
                .cmd_clear_color_image(
                    cmb[i]._buffer,
                    self._swapchain.images[i],
                    vk::ImageLayout::GENERAL,
                    &clear_value.color,
                    &[ranges])
            };

            let barrier_to_present = vk::ImageMemoryBarrier {
                old_layout: vk::ImageLayout::GENERAL,
                new_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                image: self._swapchain.images[i],
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                dst_access_mask: vk::AccessFlags::empty(),
                ..Default::default()
            };

            unsafe {
                cmb[i]._device.cmd_pipeline_barrier(
                    cmb[i]._buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[barrier_to_present],
                );
            }

            cmb[i].end()?;
        }
        let image_index:u32 = 0;
        let dev = khr::swapchain::Device::new(&self._core._instance, &self._core._logical_device);
        unsafe { dev.acquire_next_image(
            self._swapchain.swapchain,
            u64::MAX,
            image_available.clone(),
            Fence::null()).map_err(|_| "Err acquire_next_image")?
        };
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let submits = vk::SubmitInfo {
            command_buffer_count: 1,
            p_command_buffers: &cmb[image_index as usize]._buffer,
            wait_semaphore_count: 1,
            p_wait_semaphores: &image_available,
            signal_semaphore_count: 1,
            p_signal_semaphores: &render_finished,
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            ..Default::default()
        };
        unsafe {self._core._logical_device
            .queue_submit(self._core._graphics_queue, &[submits], Fence::null()).map_err(|_| "Err queue_submit")?
        };
        let present_info = vk::PresentInfoKHR {
            wait_semaphore_count: 1,
            p_wait_semaphores: &render_finished,
            swapchain_count: 1,
            p_swapchains: &self._swapchain.swapchain,
            p_image_indices: &image_index,
            ..Default::default()
        };
        
        unsafe { dev.queue_present(self._core._graphics_queue, &present_info).map_err(|_| "Err queue_present")?};

        unsafe {
            if self._core._logical_device.device_wait_idle().is_err() {
                println!("Something went wrong with the logical device wait");
            }
        }
        self._command_pool.free_buffers(&cmb);
        unsafe {
            self._core._logical_device.destroy_semaphore(image_available, None);
            self._core._logical_device.destroy_semaphore(render_finished, None);
        };
        Ok(())
    }
}

