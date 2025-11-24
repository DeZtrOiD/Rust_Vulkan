#[cfg(feature = "scene2")]
use crate::scenes::lighting::objects::LightObject;
use crate::vulkan_wr::renderable_traits::{InitFrameResources, InitObject, RenderFrameResources, RenderObject, UpdateObject};
use crate::{scenes::sphere::objects::SphereObject, vulkan_wr::ImGui_wr::VulkanImgui};
use crate::vulkan_wr::ImGui_wr::{ImguiResources, UpdateImguiResources};
use crate::vulkan_wr::renderable_traits::UpdateObjectResources;
use super::super::lighting::objects::UpdateLightObject;
pub enum RenderObjectEnum<R: ImguiResources + Default> {
    #[cfg(feature = "scene1")]
    Sphere(SphereObject),
    ImGui(VulkanImgui<R>),
    #[cfg(feature = "scene2")]
    Light(LightObject),
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
            #[cfg(feature = "scene1")]
            RenderObjectEnum::Sphere(obj) => {obj.render(app, resources)},
            #[cfg(feature = "scene2")]
            RenderObjectEnum::Light(obj) => {obj.render(app, resources)},
        }
    }
}


use super::super::sphere::update::{ResourcesSphere};
use super::super::sphere::objects::UpdateSphereObject;


#[cfg(feature = "scene1")]
impl<'a, T, R: ImguiResources + Default, Resources: UpdateObjectResources<T> + UpdateSphereObject + UpdateImguiResources<R>>
UpdateObject<T, Resources> for RenderObjectEnum<R>{
    fn update(&mut self, app: & mut crate::vulkan_wr::app::VulkanApp, resources: &mut Resources) -> Result<(), &'static str> {
        match self {
            RenderObjectEnum::ImGui(obj) => { obj.update(app, resources)},
            RenderObjectEnum::Sphere(obj) => {obj.update(app, resources)},
        }
    }
}

#[cfg(feature = "scene2")]
impl<'a, T, R: ImguiResources + Default, Resources: UpdateObjectResources<T> + UpdateLightObject + UpdateImguiResources<R>>
UpdateObject<T, Resources> for RenderObjectEnum<R>{
    fn update(&mut self, app: & mut crate::vulkan_wr::app::VulkanApp, resources: &mut Resources) -> Result<(), &'static str> {
        match self {
            RenderObjectEnum::ImGui(obj) => { obj.update(app, resources)},
            RenderObjectEnum::Light(obj) => {obj.update(app, resources)},
        }
    }
}
