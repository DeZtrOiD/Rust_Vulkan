
use std::ptr::null;

use super::{
    app::VulkanApp,
    command_pb::command_buffer::VulkanCommandBuffer,
    render_pass::pass::VulkanRenderPass,
    sync::fence::VulkanFence,
    framebuffer::VulkanFramebuffer,
};
use crate::vulkan_wr::image::image_view::VulkanImageView;
use ash::vk;

pub trait RenderObjectResources {}
pub trait InitObjectResources {}
pub trait UpdateObjectResources<T> {
    fn read(&mut self, arg: &mut T) -> Result<(), &'static str>;
    fn write(&mut self, arg: &mut T) -> Result<(), &'static str>;
}
pub trait ShutdownObjectResources {}

pub trait RenderObject<T: RenderObjectResources> {
    fn render(&mut self,
        app: & mut VulkanApp,
        resources: &T,
    ) -> Result<(), &'static str>;
}


pub trait InitObject<T: InitObjectResources> {
    type OutObject;
    fn init(app: & mut VulkanApp, resources: &mut T) -> Result<Self::OutObject, &'static str>;
}

pub trait UpdateObject<R, T: UpdateObjectResources<R>> {
    fn update(&mut self, app: & mut VulkanApp, resources: &mut T) -> Result<(), &'static str>;
}

pub trait ShutdownObject<T: ShutdownObjectResources> {
    fn shutdown(app: & mut VulkanApp, resources: &mut T) -> Result<(), &'static str>;
}

pub struct InitFrameResources<'a> {
    pub render_pass: Option<&'a VulkanRenderPass>,
    pub upload_cmd: Option<&'a VulkanCommandBuffer>,
    pub fence: Option<&'a VulkanFence>,
}

impl<'a> InitObjectResources for InitFrameResources<'a> {}

impl<'a> Default for InitFrameResources<'a> {
    fn default() -> Self {
        Self { render_pass: None, upload_cmd: None, fence: None }
    }
}

pub struct RenderFrameResources<'a> {
    pub render_pass: Option<&'a VulkanRenderPass>,
    pub framebuffer: Option<&'a VulkanFramebuffer>,
    pub color_attachment: Option<&'a VulkanImageView>,
    pub depth_attachment: Option<&'a VulkanImageView>,
    pub render_area: Option<vk::Rect2D>,
    pub clear_values: Vec<vk::ClearValue>,
}

impl<'a> RenderObjectResources for RenderFrameResources<'a>{}

impl<'a> Default for RenderFrameResources<'a> {
    fn default() -> Self {
        Self { render_pass: None, framebuffer: None, color_attachment: None, depth_attachment: None, render_area: None, clear_values: vec![] }
    }
}

