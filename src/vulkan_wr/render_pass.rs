// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: 
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use ash::{vk, Device};
use super::swapchain::{VulkanSwapchain};

struct VulkanRenderPass {
    _pass: vk::RenderPass,
    _device: Device,
}

type RPResult<T> = Result<T, &'static str>;

impl VulkanRenderPass {
    pub(super) fn try_new(device: &Device, swapchain: &VulkanSwapchain) -> RPResult<Self> {
        let color_attachment = vk::AttachmentDescription {
            format: swapchain._format,
            samples: vk::SampleCountFlags::TYPE_1,

            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE, // никогда не баловался со stencil
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        };

        let depth_attachment = vk::AttachmentDescription {
            format: swapchain._format,
            
            samples: vk::SampleCountFlags::TYPE_1,
            
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::DONT_CARE,

            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,

            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,

            ..Default::default()
        };

        let color_ref = vk::AttachmentReference { attachment: 0, layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL };
        let depth_ref = vk::AttachmentReference { attachment: 1, layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL };

        let subbpas_des = vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: 1,
            p_color_attachments: &color_ref,
            p_depth_stencil_attachment: &depth_ref,
            ..Default::default()
        };

        let dependency = vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: {
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
            }, dst_stage_mask: {
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
            }, dst_access_mask: {
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE
            }, ..Default::default()
        };

        let render_pass_info = vk::RenderPassCreateInfo {
            attachment_count: 2,
            p_attachments: [color_attachment, depth_attachment].as_ptr(),
            p_subpasses: &subbpas_des,
            p_dependencies: &dependency,

            subpass_count: 1,
            dependency_count: 1,

            ..Default::default()
        };

        let pass = unsafe { device.create_render_pass(&render_pass_info, None)
            .map_err(|_| "Err create_render_pass")?
        };

        Ok(Self {
            _pass: pass,
            _device: device.clone()
        })
    }
    
    pub fn destroy(&self) {
        unsafe { self._device.destroy_render_pass(self._pass, None) };
    }
}
