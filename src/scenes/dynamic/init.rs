use crate::{scenes::shadows::objects::ShadowsObject, vulkan_wr::app::SceneResources};

use super::{
    frame_resources::{FrameResources},
    renderable_object::RenderObjectEnum,
};
use super::super::super::vulkan_wr::{
    ImGui_wr::{VulkanImgui, ImguiResources},
    app::VulkanApp,
    renderable_traits::InitObject,
    renderable_traits::InitFrameResources,
};


pub fn init_app<R: ImguiResources + Default>(app: &mut VulkanApp, resources: &mut FrameResources<R>) -> Result<(), &'static str> {    

    resources.vec_objects.push(RenderObjectEnum::Shadows(ShadowsObject::init(
            app,
            &mut InitFrameResources {
                upload_cmd: Some(&resources.vec_cmd_primary[0]),
                fence: Some(&resources.vec_fence[0]),
                ..Default::default()
            }
        )?)
    );

    // ----- IMGUI ------
    resources.vec_objects.push(RenderObjectEnum::ImGui(VulkanImgui::<R>::init(
            app,
            &mut InitFrameResources {
                upload_cmd: Some(&resources.vec_cmd_primary[0]),
                fence: Some(&resources.vec_fence[0]),
                ..Default::default()
            }
        )?)
    );


    
    resources.init_framebuffer(app)?;

    resources.start_time = std::time::Instant::now();
    Ok(())

}
