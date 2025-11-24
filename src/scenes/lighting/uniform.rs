
// #[repr(C)] // без компилятор может поменять порядок
// #[derive(Clone, Copy, Debug)]

// pub struct MaterialUBO {
//     pub ambient: [f32; 4],   // .w свободен
//     pub diffuse: [f32; 4],
//     pub specular: [f32; 4],
//     pub extra: [f32; 4],     // extra[0] = shininess, остальные — padding
// }

#[repr(C)] // без компилятор может поменять порядок
#[derive(Clone, Copy, Debug)]
pub struct Uniforms {
    pub mvp: [[f32;4];4], // локальное в NDC 
    pub time: f32,
    pub(super) _pad: [f32;3], // выравнивание до 16 байт v4 float
}
