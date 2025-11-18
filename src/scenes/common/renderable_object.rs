use crate::vulkan_wr::renderable_traits::{InitFrameResources, InitObject, RenderFrameResources, RenderObject, UpdateObject};
use crate::{scenes::sphere::objects::SphereObject, vulkan_wr::ImGui_wr::VulkanImgui};
use crate::vulkan_wr::ImGui_wr::{ImguiResources, UpdateImguiResources};
use crate::vulkan_wr::renderable_traits::UpdateObjectResources;

pub enum RenderObjectEnum<R: ImguiResources + Default> {
    Sphere(SphereObject),
    ImGui(VulkanImgui<R>)
}

pub trait GetFrameObj<R: ImguiResources + Default> {
    fn get_frame_obj(&mut self) -> Result<&mut [RenderObjectEnum<R>], &'static str>;
    // IMGUI всегда СНОВА СНОВА ПОСЛЕДНИЙ
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


use super::super::sphere::update::{ResourcesSphere};
use super::super::sphere::objects::UpdateSphereObject;

impl<'a, R: ImguiResources + Default, Resources: UpdateObjectResources + UpdateSphereObject + UpdateImguiResources<R>>
UpdateObject<Resources> for RenderObjectEnum<R>{
    fn update(&mut self, app: & mut crate::vulkan_wr::app::VulkanApp, resources: &mut Resources) -> Result<(), &'static str> {
        match self {
            RenderObjectEnum::ImGui(obj) => { obj.update(app, resources)},
            RenderObjectEnum::Sphere(obj) => {obj.update(app, resources)},
        }
    }
}
