
use super::{
    frame_resources::FrameResources,
    uniform::Uniforms
};
use super::super::super::vulkan_wr::{
    app::VulkanApp,
    types::{matrix::Matrix, rotation_matrix::RotationMatrix},
    ImGui_wr::ImGUIUniform,
};

pub fn update_app(app: &mut VulkanApp, resources: &mut FrameResources) -> Result<(), &'static str> {
    let time = (std::time::Instant::now() - resources.start_time).as_secs_f32();

    let speed = 0.6;  // 0.6 rad/s
    let mvp_cpu = Matrix::rotation_y(time * speed).data;

    let u = Uniforms { mvp: mvp_cpu, time: time, _pad: [0.0,0.0,0.0] };

    for ub in &resources.uniform_buffers {
        unsafe {
            ub.mem_copy(&[u], None, None, None)?;
        }
    }

    Ok(())
}
