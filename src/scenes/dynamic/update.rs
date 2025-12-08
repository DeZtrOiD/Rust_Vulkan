
#[cfg(feature = "scene2")]
use crate::scenes::lighting::objects::UpdateLightObject;

#[cfg(feature = "scene3")]
use super::super::shadows::objects::UpdateShadowsObject;


use super::super::{super::vulkan_wr::{
    app::VulkanApp,
    renderable_traits::{UpdateObjectResources, UpdateObject},
    ImGui_wr::{ImguiResources, UpdateImguiResources},
}};

use super::super::sphere::objects::UpdateSphereObject;
use super::renderable_object::{GetFrameObj};
use super::frame_resources::FrameResources;

#[cfg(feature = "scene1")]
pub fn update_app<R: ImguiResources + Default, Res: UpdateObjectResources<FrameResources<R>> + UpdateImguiResources<R> + UpdateSphereObject + Default>
(app: &mut VulkanApp, resources: &mut FrameResources<R>) -> Result<(), &'static str> {

    let mut res_loc = Res::default();
    res_loc.read(resources)?;
    // IMGUI должен обработаться первым чтобы снять значения с интерфейса 
    // и сохранить их в res_loc
    for res in resources.get_frame_obj()?.iter_mut().rev() {
        res.update(app, &mut res_loc)?;
    }
    res_loc.write(resources)?;
    Ok(())
}

#[cfg(feature = "scene2")]
pub fn update_app<R: ImguiResources + Default, Res: UpdateObjectResources<FrameResources<R>> + UpdateImguiResources<R> + UpdateLightObject + Default>
(app: &mut VulkanApp, resources: &mut FrameResources<R>) -> Result<(), &'static str> {

    let mut res_loc = Res::default();
    res_loc.read(resources)?;

    // IMGUI должен обработаться первым чтобы снять значения с интерфейса 
    // и сохранить их в res_loc
    for res in resources.get_frame_obj()?.iter_mut().rev() {
        res.update(app, &mut res_loc)?;
    }
    res_loc.write(resources)?;
    Ok(())
}

#[cfg(feature = "scene3")]
pub fn update_app<R: ImguiResources + Default, Res: UpdateObjectResources<FrameResources<R>> + UpdateImguiResources<R> + UpdateShadowsObject + Default>
(app: &mut VulkanApp, resources: &mut FrameResources<R>) -> Result<(), &'static str> {

    let mut res_loc = Res::default();
    res_loc.read(resources)?;

    // IMGUI должен обработаться первым чтобы снять значения с интерфейса 
    // и сохранить их в res_loc
    for res in resources.get_frame_obj()?.iter_mut().rev() {
        res.update(app, &mut res_loc)?;
    }
    res_loc.write(resources)?;
    Ok(())
}

