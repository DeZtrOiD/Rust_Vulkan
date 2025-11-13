
use super::frame_resources::FrameResources;
use super::super::super::vulkan_wr::{
    app::VulkanApp,
};

pub fn shutdown_app(app: & mut VulkanApp, resources: &mut FrameResources) -> Result<(), &'static str> {
    Ok(())
}
