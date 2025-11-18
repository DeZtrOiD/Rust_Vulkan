
use super::super::{super::vulkan_wr::{
    app::VulkanApp,
    types::matrix::Matrix,
    renderable_traits::{
        InitObject, InitObjectResources,
        RenderObject, RenderObjectResources,
        UpdateObject, UpdateObjectResources,
        ShutdownObject, ShutdownObjectResources},
    ImGui_wr::{ImguiResources, UpdateImguiResources},
}};

use super::super::sphere::objects::UpdateSphereObject;
use super::renderable_object::{GetFrameObj, RenderObjectEnum};
use super::frame_resources::FrameResources;

pub fn update_app<R: ImguiResources + Default, Res: UpdateObjectResources + UpdateImguiResources<R> + UpdateSphereObject + Default>
(app: &mut VulkanApp, resources: &mut FrameResources<R>) -> Result<(), &'static str> {

    let mut res_loc = Res::default();

    for res in resources.get_frame_obj()? {
        match res {
            RenderObjectEnum::Sphere(sph) => {
                res_loc.update_sphere(sph, app)?;
            },
            RenderObjectEnum::ImGui(im) => {
                res_loc.update_imgui(im, app)?;
            }
        }
    }

    Ok(())
}

