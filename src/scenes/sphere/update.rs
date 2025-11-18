
use super::{
    uniform::Uniforms,
    objects::{UpdateSphereObject, SphereObject},
};
use super::super::super::vulkan_wr::{
    app::VulkanApp,
    types::{matrix::Matrix, rotation_matrix::RotationMatrix},
    ImGui_wr::{UpdateImguiResources, VulkanImgui},
    renderable_traits::UpdateObjectResources,
};
use super::frame_resources::ImguiFrameResources;

pub struct ResourcesSphere {
    mvp: Matrix<f32, 4, 4>,
    // resources: &'a FrameResources,
    aimation_time: f32,  // self.aimation_time
}

impl UpdateObjectResources for ResourcesSphere {}


impl UpdateSphereObject for ResourcesSphere {
    fn update_sphere(&mut self, obj: &mut SphereObject, app: & mut VulkanApp) -> Result<(), &'static str> {
        let u = Uniforms { mvp: self.mvp.data, time: self.aimation_time, _pad: [0.0,0.0,0.0] };

        for ub in &obj.uniform_buffers {
            unsafe {
                ub.mem_copy(&[u], None, None, None)?;
            }
        }
        Ok(())
    }
}


impl UpdateImguiResources<ImguiFrameResources> for ResourcesSphere {
    fn update_imgui(&mut self,
        imgui: &mut VulkanImgui<ImguiFrameResources>,
        app: &mut VulkanApp
    ) -> Result<(), &'static str> {

        let curr_time = std::time::Instant::now();
        if !imgui.resources.animation_paused {
            let dt = (curr_time - imgui.resources.prev_time).as_secs_f32();
            if imgui.resources.animation_reverse {
                imgui.resources.aimation_time -= dt;
            } else {
                imgui.resources.aimation_time += dt;
            }
        }
        imgui.resources.prev_time = curr_time;
        
        // Пульсация масштаба
        self.aimation_time = imgui.resources.aimation_time;
        let t = self.aimation_time;
        imgui.resources.pulse_scale = 1.0 + 0.3 * (t * 2.0).sin();

        let a = 0.8;
        let b = 0.2;
        let x = a * (t).sin();
        let y = b * (t).sin() * (t).cos();
        let offset = Matrix::translate(x, y, 0.0);

        // Матрица модели с пульсацией
        let scale_matrix = Matrix::scale(imgui.resources.pulse_scale, imgui.resources.pulse_scale, imgui.resources.pulse_scale);

        let model_matrix = offset * scale_matrix * Matrix::rotation_x(t);
        
        // Матрица вида с вращением камеры
        let view_matrix = Matrix::rotation_x(imgui.resources.camera_rotation[1]) * 
                        Matrix::rotation_y(imgui.resources.camera_rotation[0]) * Matrix::translate(0.0, 0.0, -3.0);

        // Матрица проекции
        let aspect = app.swapchain.extent.width as f32 / app.swapchain.extent.height as f32;
        let proj_matrix = if imgui.resources.use_perspective {
            Matrix::perspective(45.0f32.to_radians(), aspect, 0.1, 10.0)
        } else {
            Matrix::orthographic(-1.0, 1.0, -1.0, 1.0, 0.1, 10.0)
        };
        self.mvp = (proj_matrix * view_matrix * model_matrix).transpose();
        Ok(())
    }
}

impl Default for ResourcesSphere {
    fn default() -> Self {
        Self { mvp: Matrix::identity(), aimation_time: 0.0 }
    }
}
