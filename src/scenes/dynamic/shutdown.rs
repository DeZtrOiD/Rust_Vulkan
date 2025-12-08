
use super::frame_resources::FrameResources;
use super::super::super::vulkan_wr::{
    app::VulkanApp,
};
use crate::vulkan_wr::ImGui_wr::ImguiResources;

pub fn shutdown_app<R: ImguiResources + Default>(app: & mut VulkanApp, resources: &mut FrameResources<R>) -> Result<(), &'static str> {
    Ok(())
}
