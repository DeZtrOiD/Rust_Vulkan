
use super::super::super::vulkan_wr::{
    ImGui_wr::{ImguiResources},
};

pub struct ImguiFrameResourcesLight {
    pub animation_paused: bool,
    pub animation_reverse: bool,
    pub use_perspective: bool,
    pub camera_rotation: [f32; 2], // [yaw, pitch]
    pub pulse_scale: f32,
    pub start_time: std::time::Instant,
    pub prev_time: std::time::Instant,
    pub aimation_time: f32,
}

impl ImguiResources for ImguiFrameResourcesLight {
    fn render_ui(&mut self, ui: &mut imgui::Ui) {
        ui.window("Sphere Controls").build(|| {
            ui.text("Animation Controls:");
            
            // Переключение проекции
            ui.checkbox("Perspective Projection", &mut self.use_perspective);
            
            // Управление анимацией
            ui.checkbox("Pause Animation", &mut self.animation_paused);
            ui.checkbox("Reverse Animation", &mut self.animation_reverse);
            
            ui.separator();
            ui.text("Camera Rotation:");
            
            // Вращение камеры
            ui.slider("Yaw", -3.14, 3.14, &mut self.camera_rotation[0]);
            ui.slider("Pitch", -1.57, 1.57, &mut self.camera_rotation[1]);
            
            ui.separator();
            ui.text("Info:");
            ui.text(format!("Pulse Scale: {:.2}", self.pulse_scale));
            ui.text(format!("Time: {:.2}", (self.prev_time - self.start_time).as_secs_f32()));
        });

    }
}

impl Default for ImguiFrameResourcesLight {
    fn default() -> Self {
        let time = std::time::Instant::now();
        Self { 
            animation_paused: false,
            animation_reverse: false,
            use_perspective: true,
            camera_rotation: [0.0, 0.0],
            pulse_scale: 1.0,
            start_time: time,
            prev_time: time,
            aimation_time: 0.0,
        }
    }
}

