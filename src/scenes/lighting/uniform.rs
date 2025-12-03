
#[repr(C)] // без компилятор может поменять порядок
#[derive(Clone, Copy, Debug)]
pub struct Uniforms {
    pub view_proj: [[f32;4];4], // локальное в NDC 
    // pub world: [[f32;4];4], // локальное в NDC
    pub camera: [f32; 4],
    pub time: f32,
    pub(super) _pad: [f32;3], // выравнивание до 16 байт v4 float
}

#[repr(C, align(16))] // без компилятор может поменять порядок
#[derive(Clone, Copy, Debug)]
pub struct LightsSSBO {
    pub light_count_directional: u32,
    pub light_count_point: u32,
    pub light_count_spotlight: u32,
    pub _pad: u32,

    pub directional_lights: [DirectionalLight; 5],
    pub point_lights: [PointLight; 5],
    pub spotlights: [Spotlight; 5],
}


#[repr(C, align(8))] // без компилятор может поменять порядок
#[derive(Clone, Copy, Debug)]
pub struct DirectionalLight {
    pub direction: [f32; 4], // .w свободен
    pub color: [f32; 4],     // .w = intensity
}


#[repr(C, align(16))] // без компилятор может поменять порядок
#[derive(Clone, Copy, Debug)]
pub struct Spotlight {
    pub position: [f32; 4],  // .w свободен
    pub direction: [f32; 4], // .w = cutoff angle in radians
    pub color: [f32; 4],     // .w = intensity
    pub cut_off: [f32; 4],
    // pub _pad: [f32; 3],      // выравнивание до 16 байт v4 float
}

// ssbo требует выравние для массивов даже в std430. не чет другое
#[repr(C, align(16))] // без компилятор может поменять порядок
#[derive(Clone, Copy, Debug)]
pub struct PointLight {
    pub position: [f32; 4], // .w свободен
    pub color: [f32; 4],    // .w = intensity
    pub coefficients: [f32; 4],
    pub _pad: [f32; 4],     // выравнивание до 16 байт v4 float
    // pub _pad1: [f32; 4],     // выравнивание до 16 байт v4 float
}

impl Default for LightsSSBO {
    fn default() -> Self {
        Self {
            light_count_directional: 0,
            light_count_point: 0,
            light_count_spotlight: 0,
            _pad: 0,
            directional_lights: [DirectionalLight {..Default::default()}; 5],
            point_lights: [PointLight {..Default::default()}; 5],
            spotlights: [Spotlight {..Default::default()}; 5],
        }
    }
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: [0.0; 4],
            color: [0.0; 4],
        }
    }
}

impl Default for Spotlight {
    fn default() -> Self {
        Self {
            position: [0.0; 4],
            direction: [0.0; 4],
            color: [0.0; 4],
            cut_off: [1.0; 4],
            // _pad: [0.0; 3],
        }
    }
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            position: [0.0; 4],
            color: [0.0; 4],
            coefficients: [1.0; 4],
            _pad: [0.0; 4],
            // _pad1: [0.0; 4],
        }
    }
}
