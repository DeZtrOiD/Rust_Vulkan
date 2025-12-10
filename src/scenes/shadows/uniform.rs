
pub const MAX_LIGHTS_IN_CAT: usize = 5;


#[repr(C)] // без компилятор может поменять порядок
#[derive(Clone, Copy, Debug)]
pub struct Uniforms {
    pub view_proj: [[f32;4];4], // локальное в NDC 
    // pub world: [[f32;4];4], // локальное в NDC
    pub camera: [f32; 4],
    pub time: f32,
    pub(super) _pad: [f32;3], // выравнивание до 16 байт v4 float
}

pub struct ShadowsUniform {
    pub light_space_matrix: [[f32;4];4], // локальное в NDC
    pub indx: u32,
    pub _pad: [f32; 3],
}

impl Default for ShadowsUniform {
    fn default() -> Self {
        Self { light_space_matrix: [[0.0; 4]; 4], indx: 0, _pad: [0.0; 3] }
    }
}

#[repr(C, align(16))] // без компилятор может поменять порядок
#[derive(Clone, Copy, Debug)]
pub struct LightsSSBO {
    pub light_count_directional: u32,
    pub light_count_point: u32,
    pub light_count_spotlight: u32,
    pub time: f32,
    // pub _pad: u32,

    pub directional_lights: [DirectionalLight; MAX_LIGHTS_IN_CAT],
    pub point_lights: [PointLight; MAX_LIGHTS_IN_CAT],
    pub spotlights: [Spotlight; MAX_LIGHTS_IN_CAT],


    // pub directional_light_matrices: [[[f32; 4]; 4]; 5], // ViewProj для каждого направленного света
    // pub point_light_far_planes: [f32; 5], // Far plane для точечных источников
    pub _pad: [f32; 3], // Выравнивание
}


#[repr(C, align(8))] // без компилятор может поменять порядок
#[derive(Clone, Copy, Debug)]
pub struct DirectionalLight {
    pub direction: [f32; 4], // .w свободен
    pub color: [f32; 4],     // .w = intensity
    pub light_matrices: [[f32; 4]; 4],
}


#[repr(C, align(16))] // без компилятор может поменять порядок
#[derive(Clone, Copy, Debug)]
pub struct Spotlight {
    pub position: [f32; 4],  // .w свободен
    pub direction: [f32; 4], // .w = cutoff angle in radians
    pub color: [f32; 4],     // .w = intensity
    pub cut_off: [f32; 4],
    // pub _pad: [f32; 3],      // выравнивание до 16 байт v4 float
    pub light_matrices: [[f32; 4]; 4],
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
    pub light_matrices: [[f32; 4]; 4],
}

impl Default for LightsSSBO {
    fn default() -> Self {
        Self {
            light_count_directional: 0,
            light_count_point: 0,
            light_count_spotlight: 0,
            time: 0.0,
            // _pad: 0,
            directional_lights: [DirectionalLight {..Default::default()}; MAX_LIGHTS_IN_CAT],
            point_lights: [PointLight {..Default::default()}; MAX_LIGHTS_IN_CAT],
            spotlights: [Spotlight {..Default::default()}; MAX_LIGHTS_IN_CAT],
            // directional_light_matrices: [[[0.0; 4]; 4]; 5],
            // point_light_far_planes: [0.0; 5],
            _pad: [0.0; 3],
        
        }
    }
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: [0.0; 4],
            color: [0.0; 4],
            light_matrices: [[0.0; 4]; 4]
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
            light_matrices: [[0.0; 4]; 4]
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
            light_matrices: [[0.0; 4]; 4]
            // _pad1: [0.0; 4],
        }
    }
}
