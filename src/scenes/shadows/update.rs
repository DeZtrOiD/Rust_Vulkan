
use std::{f32::consts::PI, mem::offset_of};

use crate::{scenes::lighting::uniform::{DirectionalLight, LightsSSBO, PointLight, Spotlight}, vulkan_wr::types::vector::VulkanVector};

use super::{
    uniform::Uniforms,
    objects::{UpdateShadowsObject, ShadowsObject},
};
use super::super::super::vulkan_wr::{
    app::VulkanApp,
    types::{matrix::Matrix},
    ImGui_wr::{UpdateImguiResources, VulkanImgui},
    renderable_traits::UpdateObjectResources,
};
use super::frame_resources::{ImguiFrameResourcesShadows};
use super::super::super::window::{KEY_CODES, key_to_index};
use super::super::dynamic::frame_resources::{FrameResources, Camera};
use crate::vulkan_wr::ImGui_wr::ImguiResources;

pub struct ResourcesShadows {
    mvp: Matrix<4, 4>,
    // resources: &'a FrameResources,
    animation_time: f32,  // self.aimation_time
    key_w: bool,
    key_a: bool,
    key_s: bool,
    key_d: bool,
    key_alt: bool,
    rotation: f32,
    coefficient_linear: f32,
    coefficient_quadratic: f32,
    radius_spotlight: f32,
    outer_cut_off: f32,
    inner_cut_off: f32,
    mouse_pos: [f32; 2],
    mouse_down: [bool; 5],
    mouse_delta: [f32; 2],
    light_count_directional: u32,
    light_count_point: u32,
    light_count_spotlight: u32,
    camera: Camera,
    time: f32,
}

impl<R: ImguiResources + Default> UpdateObjectResources<FrameResources<R>> for ResourcesShadows {
    fn read(&mut self, arg: &mut FrameResources<R>) -> Result<(), &'static str> {
        self.camera = arg.camera;
        Ok(())
    }
    fn write(&mut self, arg: &mut FrameResources<R>) -> Result<(), &'static str> {
        arg.camera = self.camera;
        Ok(())
    }
}

impl UpdateShadowsObject for ResourcesShadows {
    fn update_shadows(&mut self, obj: &mut ShadowsObject, app: & mut VulkanApp) -> Result<(), &'static str> {
        const SPEED: f32 = 0.2;
        const SENSITIVITY: f32 = 0.002;

        let t = self.animation_time;

        if !self.key_alt {
            self.camera.update(self.mouse_delta, SENSITIVITY);
        }

        // да, это странно выглядит, но это лучше всего работало на плюсах
        // как минимум проверки на то что клавиши нажаты одновременно
        // противоположные, стоит сделать
        if (self.key_w != self.key_s) || (self.key_a != self.key_d) {
            if self.key_w != self.key_s {
                let forward = self.camera.forward() * SPEED;
                if self.key_w {
                    self.camera.pos += forward;
                } else {
                    self.camera.pos -= forward;
                }
            }
            if self.key_a != self.key_d {
                let right = self.camera.right() * SPEED;
                if self.key_a {
                    self.camera.pos -= right;
                } else {
                    self.camera.pos += right;
                }
            }
            self.camera.dirty = true;
        }

        // Матрица проекции
        let aspect = app.swapchain.extent.width as f32 / app.swapchain.extent.height as f32;
        let proj_matrix = if true {
            Matrix::perspective(45.0f32.to_radians(), aspect, 0.1, 100.0)
        } else {
            let h = 1.0;
            let w = h * aspect;
            Matrix::orthographic(-w, w, -h, h, 0.1, 100.0)
        };
        self.mvp = (proj_matrix * self.camera.view_matrix()?).transpose();

        let u = Uniforms {
            view_proj: self.mvp.data,
            // world: model_matrix.transpose().data,
            time: self.animation_time,
            _pad: [0.0,0.0,0.0],
            camera: [self.camera.pos[0], self.camera.pos[1], self.camera.pos[2], 0.0]
        };

        for ub in &obj.uniform_buffers {
            unsafe {
                ub.mem_copy(&[u], None, None, None)?;
            }
        }
        
        let ssbo = LightsSSBO {
            time: self.time,
            light_count_directional: self.light_count_directional,
            light_count_point: self.light_count_point,
            light_count_spotlight: self.light_count_spotlight,
            directional_lights: [
                DirectionalLight { // blue left
                    direction: [-1.0, self.rotation / PI, 0.0, 0.0],  // [0.5, 1.0, 0.3, 0.0]
                    color: [1.0, 0.0, 0.0, 1.0],
                },
                DirectionalLight { // red right
                    direction: [1.0, self.rotation / PI, 0.0, 0.0],
                    color: [0.0, 1.0, 0.0, 1.0],
                },
                DirectionalLight {  // green forward
                    direction: [0.0, self.rotation / PI, -1.0, 0.0],
                    color: [0.0, 0.0, 1.0, 1.0],
                },
                DirectionalLight { // white back
                    direction: [0.0, self.rotation / PI, 1.0, 0.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                },
                DirectionalLight {
                    direction: [0.5, 1.0, 0.3, 0.0],
                    color: [0.0, 0.0, 0.0, 0.0],
                }
            ],
            point_lights: [
                PointLight {
                    position: [ 4.0, -4.0, 4.0, 0.0],
                    color: [ 1.0, 0.0, 0.0, 1.0],
                    coefficients: [1.0, self.coefficient_linear, self.coefficient_quadratic, 0.0],
                    ..Default::default()},
                PointLight {
                    position: [-4.0, -4.0, 4.0, 0.0],
                    color: [ 0.0, 1.0, 0.0, 1.0],
                    coefficients: [1.0, self.coefficient_linear, self.coefficient_quadratic, 0.0],                    
                    ..Default::default()},
                PointLight {
                    position: [ 4.0, -4.0, -4.0, 0.0],
                    color: [ 0.0, 0.0, 1.0, 1.0],
                    coefficients: [1.0, self.coefficient_linear, self.coefficient_quadratic, 0.0],
                    ..Default::default()},
                PointLight {
                    position: [-4.0, -4.0, -4.0, 0.0],
                    color: [ 1.0, 1.0, 1.0, 1.0],
                    coefficients: [1.0, self.coefficient_linear, self.coefficient_quadratic, 0.0],
                    ..Default::default()},
                PointLight {..Default::default()}
            ],
            spotlights: [
                Spotlight {
                    position: [9.0, -1.0, 0.0, 1.0],
                    direction: [-1.0, 0.0, 0.0, 0.0 ],
                    color: [1.0, 0.0, 0.0, 10.0],
                    cut_off: [((self.outer_cut_off).to_radians()).cos(), ((self.inner_cut_off).to_radians()).cos(), 1.0, 1.0],
                    ..Default::default()},
                Spotlight {
                    position: [ -9.0, -1.0, 0.0, 1.0],
                    direction: [1.0, 0.0, 0.0, 0.0 ],
                    color: [ 0.0, 1.0, 0.0, 10.0],
                    cut_off: [((self.outer_cut_off).to_radians()).cos(), ((self.inner_cut_off).to_radians()).cos(), 1.0, 1.0],
                    ..Default::default()},
                Spotlight {
                    position: [ 0.0, -1.0, 9.0, 1.0],
                    direction: [ 0.0, 0.0, -1.0, 0.0 ],
                    color: [ 0.0, 0.0, 1.0, 10.0],
                    cut_off: [((self.outer_cut_off).to_radians()).cos(), ((self.inner_cut_off).to_radians()).cos(), 1.0, 1.0],
                    ..Default::default()},

                Spotlight {
                    position: [ 0.0, -1.0,-9.0, 1.0],
                    direction: [ 0.0, 0.0, 1.0, ((self.outer_cut_off).to_radians()).cos() ],
                    color: [ 1.0, 1.0, 1.0, 10.0],
                    cut_off: [((self.outer_cut_off).to_radians()).cos(), ((self.inner_cut_off).to_radians()).cos(), 1.0, 1.0],

                    ..Default::default()},
                Spotlight {..Default::default()}
            ],
            ..Default::default()
        };
        for sb in &obj.ssbo_light_buffer {
            unsafe {
                sb.mem_copy(&[ssbo], None, None, None)?;
            }
        }

        Ok(())
    }
}

impl UpdateImguiResources<ImguiFrameResourcesShadows> for ResourcesShadows {
    fn update_imgui(&mut self,
        imgui: &mut VulkanImgui<ImguiFrameResourcesShadows>,
        app: &mut VulkanApp
    ) -> Result<(), &'static str> {

        
        let curr_time = std::time::Instant::now();

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
            self.key_alt = io.key_alt;
        }
        self.light_count_directional = imgui.resources.light_count_directional;
        self.light_count_point = imgui.resources.light_count_point;
        self.light_count_spotlight = imgui.resources.light_count_spotlight;
        self.rotation = imgui.resources.rotation;
        self.coefficient_linear = imgui.resources.coefficient_linear;
        self.coefficient_quadratic =imgui.resources.coefficient_quadratic;
        self.radius_spotlight = imgui.resources.radius_spotlight;
        self.outer_cut_off = imgui.resources.outer_cut_off;
        self.inner_cut_off = imgui.resources.inner_cut_off;
        self.time = (imgui.resources.prev_time - imgui.resources.start_time).as_secs_f32();

        Ok(())
    }
}

impl Default for ResourcesShadows {
    fn default() -> Self {
        Self {
            mvp: Matrix::identity(),
            animation_time: 0.0,
            key_w: false,
            key_a: false,
            key_s: false,
            key_d: false,
            key_alt: false,
            mouse_delta: [0.0, 0.0],
            mouse_pos: [0.0, 0.0],
            mouse_down: [false; 5],
            camera: Camera::default(),
            light_count_directional: 0,
            light_count_point: 0,
            light_count_spotlight: 0,
            rotation: 0.0,
            coefficient_linear: 0.25,
            radius_spotlight: 10.0,
            outer_cut_off: 20.0,
            coefficient_quadratic: 0.2,
            inner_cut_off: 20.0,
            time: 0.0,
        }
    }
}
