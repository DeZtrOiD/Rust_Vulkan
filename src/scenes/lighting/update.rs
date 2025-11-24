
use std::{f32::consts::PI, mem::offset_of};

use crate::vulkan_wr::types::vector::VulkanVector;

use super::{
    uniform::Uniforms,
    objects::{UpdateLightObject, LightObject},
};
use super::super::super::vulkan_wr::{
    app::VulkanApp,
    types::{matrix::Matrix},
    ImGui_wr::{UpdateImguiResources, VulkanImgui},
    renderable_traits::UpdateObjectResources,
};
use super::frame_resources::{ImguiFrameResourcesLight};
use super::super::super::window::{KEY_CODES, key_to_index};
use super::super::common::frame_resources::{FrameResources, Camera};
use crate::vulkan_wr::ImGui_wr::ImguiResources;

pub struct ResourcesLight {
    mvp: Matrix<4, 4>,
    // resources: &'a FrameResources,
    animation_time: f32,  // self.aimation_time
    key_w: bool,
    key_a: bool,
    key_s: bool,
    key_d: bool,
    key_ctrl: bool,
    mouse_pos: [f32; 2],
    mouse_down: [bool; 5],
    mouse_delta: [f32; 2],
    pulse_scale: f32,
    camera_rotation: [f32; 2],
    use_perspective: bool,
    camera: Camera,
}

impl<R: ImguiResources + Default> UpdateObjectResources<FrameResources<R>> for ResourcesLight {
    fn read(&mut self, arg: &mut FrameResources<R>) -> Result<(), &'static str> {
        self.camera = arg.camera;
        Ok(())
    }
    fn write(&mut self, arg: &mut FrameResources<R>) -> Result<(), &'static str> {
        arg.camera = self.camera;
        Ok(())
    }
}

impl UpdateLightObject for ResourcesLight {
    fn update_light(&mut self, obj: &mut LightObject, app: & mut VulkanApp) -> Result<(), &'static str> {
        const speed: f32 = 0.2;
        const sensitivity: f32 = 0.002;

        let t = self.animation_time;

        if !self.key_ctrl {
            self.camera.update(self.mouse_delta, sensitivity);
        }

        // да, это странно выглядит, но это лучше всего работало на плюсах
        // как минимум проверки на то что клавиши нажаты одновременно
        // противоположные, стоит сделать
        if (self.key_w != self.key_s) || (self.key_a != self.key_d) {
            if self.key_w != self.key_s {
                let forward = self.camera.forward() * speed;
                if self.key_w {
                    self.camera.pos += forward;
                } else {
                    self.camera.pos -= forward;
                }
            }
            if self.key_a != self.key_d {
                let right = self.camera.right() * speed;
                if self.key_a {
                    self.camera.pos -= right;
                } else {
                    self.camera.pos += right;
                }
            }
            self.camera.dirty = true;
        }

        let model_matrix =Matrix::translate_vec(&VulkanVector::new([0.0, 2.0, -6.0])) *
            Matrix::rotation_vec(&VulkanVector::new([PI/2.0, 0.0, 0.0])) *
            Matrix::identity();

        let mut view_matrix = self.camera.view_matrix()?;

        // Матрица проекции
        let aspect = app.swapchain.extent.width as f32 / app.swapchain.extent.height as f32;
        let proj_matrix = if self.use_perspective {
            Matrix::perspective(45.0f32.to_radians(), aspect, 0.1, 100.0)
        } else {
            let h = 1.0;
            let w = h * aspect;
            Matrix::orthographic(-w, w, -h, h, 0.1, 100.0)
        };
        self.mvp = (proj_matrix * view_matrix * model_matrix).transpose();

        let u = Uniforms { mvp: self.mvp.data, time: self.animation_time, _pad: [0.0,0.0,0.0] };

        for ub in &obj.uniform_buffers {
            unsafe {
                ub.mem_copy(&[u], None, None, None)?;
            }
        }
        Ok(())
    }
}

impl UpdateImguiResources<ImguiFrameResourcesLight> for ResourcesLight {
    fn update_imgui(&mut self,
        imgui: &mut VulkanImgui<ImguiFrameResourcesLight>,
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
        {
            let io = imgui.context.io_mut();
            self.key_w = io.keys_down[key_to_index(glfw::Key::W).unwrap()];
            self.key_a = io.keys_down[key_to_index(glfw::Key::A).unwrap()];
            self.key_s = io.keys_down[key_to_index(glfw::Key::S).unwrap()];
            self.key_d = io.keys_down[key_to_index(glfw::Key::D).unwrap()];
            self.mouse_pos = io.mouse_pos;
            self.mouse_down = io.mouse_down;
            self.mouse_delta = io.mouse_delta;
            self.key_ctrl = io.key_ctrl;
        }

        // Пульсация масштаба
        self.animation_time = imgui.resources.aimation_time;
        imgui.resources.pulse_scale = 1.0 + 0.3 * (self.animation_time * 2.0).sin();
        self.pulse_scale = imgui.resources.pulse_scale;
        self.use_perspective = imgui.resources.use_perspective;

        Ok(())
    }
}

impl Default for ResourcesLight {
    fn default() -> Self {
        Self {
            mvp: Matrix::identity(),
            animation_time: 0.0,
            key_w: false,
            key_a: false,
            key_s: false,
            key_d: false,
            key_ctrl: false,
            mouse_delta: [0.0, 0.0],
            mouse_pos: [0.0, 0.0],
            mouse_down: [false; 5],
            pulse_scale: 0.0,
            camera_rotation: [0.0; 2],
            use_perspective: false,
            camera: Camera::default(),
        }
    }
}
