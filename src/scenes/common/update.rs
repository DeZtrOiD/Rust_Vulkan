
use super::super::{super::vulkan_wr::{
    app::VulkanApp,
    renderable_traits::{UpdateObjectResources, UpdateObject},
    ImGui_wr::{ImguiResources, UpdateImguiResources},
}};

use super::super::sphere::objects::UpdateSphereObject;
use super::renderable_object::{GetFrameObj};
use super::frame_resources::FrameResources;

pub fn update_app<R: ImguiResources + Default, Res: UpdateObjectResources + UpdateImguiResources<R> + UpdateSphereObject + Default>
(app: &mut VulkanApp, resources: &mut FrameResources<R>) -> Result<(), &'static str> {

    let mut res_loc = Res::default();

    // IMGUI должен обработаться первым чтобы снять значения с интерфейса 
    // и сохранить их в res_loc
    for res in resources.get_frame_obj()?.iter_mut().rev() {
        res.update(app, &mut res_loc)?;
    }

    Ok(())
}

