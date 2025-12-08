
use std::f32::consts::PI;

use super::super::super::vulkan_wr::{
    ImGui_wr::{ImguiResources},
};

pub struct ImguiFrameResourcesShadows {

    pub start_time: std::time::Instant,
    pub prev_time: std::time::Instant,
    pub light_count_directional: u32,
    pub light_count_point: u32,
    pub light_count_spotlight: u32,
    pub rotation: f32,
    pub coefficient_linear: f32,
    pub coefficient_quadratic: f32,
    pub radius_spotlight: f32,
    pub outer_cut_off: f32,
    pub inner_cut_off: f32,
}

impl ImguiResources for ImguiFrameResourcesShadows {
    fn render_ui(&mut self, ui: &mut imgui::Ui) {
        ui.window("Sphere Controls").build(|| {
            ui.text("Animation Controls:");
            ui.slider("Directional", 0, 4, &mut self.light_count_directional);
            ui.slider("rotation", -PI, PI, &mut self.rotation);
            ui.separator();
            ui.slider("Point", 0, 4, &mut self.light_count_point);
            ui.slider("Linear coefficient", 0.0, 2.0, &mut self.coefficient_linear);
            ui.slider("quadratic coefficient", 0.0, 2.0, &mut self.coefficient_quadratic);
            ui.separator();
            ui.slider("Spotlight", 0, 4, &mut self.light_count_spotlight);
            // ui.slider("Radius spotlight", 0.0, 120., &mut self.radius_spotlight);
            ui.slider("Outer cone", 0.0, 90.0, &mut self.outer_cut_off);
            ui.slider("Inner cone", 0.0, self.outer_cut_off, &mut self.inner_cut_off);
            
            ui.separator();
            ui.text("Info:");
            ui.text(format!("Time: {:.2}", (self.prev_time - self.start_time).as_secs_f32()));
        });

    }
}

impl Default for ImguiFrameResourcesShadows {
    fn default() -> Self {
        let time = std::time::Instant::now();
        Self { 
            start_time: time,
            prev_time: time,
            light_count_directional: 1,
            light_count_point: 0,
            light_count_spotlight: 0,
            rotation: 0.0,
            coefficient_linear: 0.25,
            radius_spotlight: 10.0,
            outer_cut_off: 20.0,
            coefficient_quadratic: 0.2,
            inner_cut_off: 20.0,
        }
    }
}

