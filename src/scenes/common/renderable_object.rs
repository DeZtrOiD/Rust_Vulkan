use crate::vulkan_wr::renderable_traits::{InitFrameResources, InitObject, RenderFrameResources, RenderObject};
use crate::{scenes::sphere::objects::SphereObject, vulkan_wr::ImGui_wr::VulkanImgui};
use crate::vulkan_wr::ImGui_wr::ImguiResources;


pub enum RenderObjectEnum<R: ImguiResources + Default> {
    Sphere(SphereObject),
    ImGui(VulkanImgui<R>)
}

pub trait GetFrameObj<R: ImguiResources + Default> {
    fn get_frame_obj(&mut self) -> Result<&mut [RenderObjectEnum<R>], &'static str>;
    // IMGUI всегда 0й
    fn get_imgui(&mut self) -> Result<&mut RenderObjectEnum<R>, &'static str>;
}

impl<'a, R: ImguiResources + Default> RenderObject<RenderFrameResources<'a>> for RenderObjectEnum<R> {
    fn render(&mut self,
            app: & mut crate::vulkan_wr::app::VulkanApp,
            resources: &RenderFrameResources,
        ) -> Result<(), &'static str> {
        match self {
            RenderObjectEnum::ImGui(obj) => {obj.render(app, resources)},
            RenderObjectEnum::Sphere(obj) => {obj.render(app, resources)},   
        }
    }
}

