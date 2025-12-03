// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: Небольшая обертка для матриц ROW-major хранение
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#

use std::{convert::identity, fmt, ops::{Add, Index, IndexMut, Mul, Sub}, process::Output};

use crate::vulkan_wr::types::vector::VulkanVector;

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Matrix<const ROWS: usize, const COLS: usize> {
    pub data: [[f32; COLS]; ROWS]
}

impl<const ROWS: usize, const COLS: usize> Matrix<ROWS, COLS> {
    pub fn new(data: [[f32; COLS]; ROWS]) -> Self {
        Matrix { data }
    }

    pub fn rows(&self) -> usize { ROWS }

    pub fn cols(&self) -> usize { COLS }

    pub fn get(&self, row: usize, col: usize) -> Option<&f32> {
        if row < ROWS && col < COLS {
            Some(&self[row][col])
        } else {
            None
        }
    }

    pub fn transpose(&self) -> Matrix<COLS, ROWS> {
        let mut result_data = [[self[0][0]; ROWS]; COLS];
        for i in 0..ROWS {
            for j in 0..COLS {
                result_data[j][i] = self[i][j];
            }
        }
        Matrix::new(result_data)
    }
}

impl<const SIZE: usize> Matrix<SIZE, SIZE> {
    pub fn identity() -> Self {
        let mut data = [[0.0; SIZE]; SIZE];
        for i in 0..SIZE {
            data[i][i] = 1.0;
        }
        Matrix::new(data)
    }
}

impl Matrix<4 , 4> {
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let (l, r, b, t, n, f) = (left, right, bottom, top, near, far);
        Matrix::new([
            [ 2.0 / (r - l),            0.0,            0.0, -(r + l) / (r - l) ],
            [           0.0,  2.0 / (t - b),            0.0, -(t + b) / (t - b) ],
            [           0.0,            0.0, -2.0 / (f - n),     -(n) / (f - n) ], // для вулкана не (f + n) тк клипинг не в [-1,1] а в [0, 1]
            [           0.0,            0.0,            0.0,                1.0 ],
        ])
        // P = Scale * Translate
    }

    //https://learnwebgl.brown37.net/08_projections/projections_perspective.html
    pub fn perspective(fov_rad: f32, aspect: f32, near: f32, far: f32) -> Self {
        let s = 1.0 / (fov_rad / 2.0).tan();  // scale
        let a = aspect; // w/h
        let fnf = far / (- far + near);

        Matrix::new([
            [   s, 0.0,  0.0,          0.0 ],
            [ 0.0, s*a,  0.0,          0.0 ],
            [ 0.0, 0.0,  fnf, 2.0*near*fnf ],
            [ 0.0, 0.0, -1.0,          0.0 ],
        ])
    }

    pub fn scale(sx: f32, sy: f32, sz: f32) -> Self {
        let mut data = [[0.0; 4]; 4];
        data[0][0] = sx;
        data[1][1] = sy;
        data[2][2] = sz;
        data[3][3] = 1.0;
        Self { data }
    }

    pub fn scale_vec(sc: &VulkanVector<3>) -> Self {
        let mut data = [[0.0; 4]; 4];
        data[0][0] = sc[0];
        data[1][1] = sc[1];
        data[2][2] = sc[2];
        data[3][3] = 1.0;
        Self { data }
    }

    pub fn translate(x: f32, y: f32, z: f32) -> Self {
        Matrix::new([
            [1.0, 0.0, 0.0, x],
            [0.0, 1.0, 0.0, y],
            [0.0, 0.0, 1.0, z],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn translate_vec(vect: &VulkanVector<3>) -> Self {
        Matrix::new([
            [1.0, 0.0, 0.0, vect[0]],
            [0.0, 1.0, 0.0, vect[1]],
            [0.0, 0.0, 1.0, vect[2]],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn rotation_axis(axis: VulkanVector<3>, angle_rad: f32) -> Result<Matrix<4, 4>, &'static str> {
        let r3 = Matrix::<3, 3>::rotation_axis(axis, angle_rad)?;
        // Встраиваем 3x3 в 4x4
        let mut m = Matrix::identity();
        for i in 0..3 {
            for j in 0..3 {
                m[i][j] = r3[i][j];
            }
        }
        Ok(m)
    }

    pub fn rotation_x(angle_rad: f32) -> Matrix<4, 4> {
        let cos = angle_rad.cos();
        let sin = angle_rad.sin();
        
        Matrix::new([
            [  1.0,  0.0,  0.0,  0.0],
            [  0.0,  cos, -sin,  0.0],
            [  0.0,  sin,  cos,  0.0],
            [  0.0,  0.0,  0.0,  1.0],
        ])
    }
    
    pub fn rotation_y(angle_rad: f32) -> Matrix<4, 4> {
        let cos = angle_rad.cos();
        let sin = angle_rad.sin();
        
        Matrix::new([
            [  cos,  0.0,  sin,  0.0],
            [  0.0,  1.0,  0.0,  0.0],
            [ -sin,  0.0,  cos,  0.0],
            [  0.0,  0.0,  0.0,  1.0],
        ])
    }
    
    pub fn rotation_z(angle_rad: f32) -> Matrix<4, 4> {
        let cos = angle_rad.cos();
        let sin = angle_rad.sin();
        
        Matrix::new([
            [  cos,  -sin,  0.0,  0.0],
            [  sin,  cos,  0.0,  0.0],
            [  0.0,  0.0,  1.0,  0.0],
            [  0.0,  0.0,  0.0,  1.0],
        ])
    }

    pub fn rotation_vec(vec_rad: &VulkanVector<3>) -> Matrix<4, 4> {
        Matrix::<4,4>::rotation_z(vec_rad[2]) *
        Matrix::<4,4>::rotation_y(vec_rad[1]) *
        Matrix::<4,4>::rotation_x(vec_rad[0])
    }

    pub fn look_at(pos: &VulkanVector<3>, target: &VulkanVector<3>, up: &VulkanVector<3>) -> Result<Self, &'static str> {
        let forward = (*target - *pos).normalize()?;
        let right = forward.cross(up).normalize()?;
        let up_ = right.cross(&forward);

        let dot_r = right.dot(pos);
        let dot_u = up_.dot(pos);
        let dot_f = forward.dot(pos);

        Ok(Matrix::new([
            [    right[0],    right[1],    right[2], -dot_r ],
            [      up_[0],      up_[1],      up_[2], -dot_u ],
            [ -forward[0], -forward[1], -forward[2],  dot_f ],
            [              0.0,              0.0,              0.0,    1.0 ],
        ]))
        // LA = B(R, U, D - rows) * T(-pos), B - Transformation matrix (change basis)
    }

    pub fn inverse(&self) -> Result<Self, &'static str> {
        let m = &self.data;

        let det = 
            m[0][0] * (
                m[1][1] * (m[2][2] * m[3][3] - m[2][3] * m[3][2])
            - m[1][2] * (m[2][1] * m[3][3] - m[2][3] * m[3][1])
            + m[1][3] * (m[2][1] * m[3][2] - m[2][2] * m[3][1])
            )
        - m[0][1] * (
                m[1][0] * (m[2][2] * m[3][3] - m[2][3] * m[3][2])
            - m[1][2] * (m[2][0] * m[3][3] - m[2][3] * m[3][0])
            + m[1][3] * (m[2][0] * m[3][2] - m[2][2] * m[3][0])
            )
        + m[0][2] * (
                m[1][0] * (m[2][1] * m[3][3] - m[2][3] * m[3][1])
            - m[1][1] * (m[2][0] * m[3][3] - m[2][3] * m[3][0])
            + m[1][3] * (m[2][0] * m[3][1] - m[2][1] * m[3][0])
            )
        - m[0][3] * (
                m[1][0] * (m[2][1] * m[3][2] - m[2][2] * m[3][1])
            - m[1][1] * (m[2][0] * m[3][2] - m[2][2] * m[3][0])
            + m[1][2] * (m[2][0] * m[3][1] - m[2][1] * m[3][0])
            );

        if det.abs() < 1e-12 {
            return Err("Matrix is singular (determinant is zero)");
        }

        let inv_det = 1.0 / det;

        let inv = [
            // row 0
            [
                (m[1][1] * (m[2][2] * m[3][3] - m[2][3] * m[3][2]) -
                m[1][2] * (m[2][1] * m[3][3] - m[2][3] * m[3][1]) +
                m[1][3] * (m[2][1] * m[3][2] - m[2][2] * m[3][1])) * inv_det,

                (m[0][2] * (m[2][1] * m[3][3] - m[2][3] * m[3][1]) -
                m[0][1] * (m[2][2] * m[3][3] - m[2][3] * m[3][2]) -
                m[0][3] * (m[2][1] * m[3][2] - m[2][2] * m[3][1])) * inv_det,

                (m[0][1] * (m[1][2] * m[3][3] - m[1][3] * m[3][2]) -
                m[0][2] * (m[1][1] * m[3][3] - m[1][3] * m[3][1]) +
                m[0][3] * (m[1][1] * m[3][2] - m[1][2] * m[3][1])) * inv_det,

                (m[0][2] * (m[1][1] * m[2][3] - m[1][3] * m[2][1]) -
                m[0][1] * (m[1][2] * m[2][3] - m[1][3] * m[2][2]) -
                m[0][3] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])) * inv_det,
            ],
            // row 1
            [
                (m[1][2] * (m[2][0] * m[3][3] - m[2][3] * m[3][0]) -
                m[1][0] * (m[2][2] * m[3][3] - m[2][3] * m[3][2]) -
                m[1][3] * (m[2][0] * m[3][2] - m[2][2] * m[3][0])) * inv_det,

                (m[0][0] * (m[2][2] * m[3][3] - m[2][3] * m[3][2]) -
                m[0][2] * (m[2][0] * m[3][3] - m[2][3] * m[3][0]) +
                m[0][3] * (m[2][0] * m[3][2] - m[2][2] * m[3][0])) * inv_det,

                (m[0][2] * (m[1][0] * m[3][3] - m[1][3] * m[3][0]) -
                m[0][0] * (m[1][2] * m[3][3] - m[1][3] * m[3][2]) -
                m[0][3] * (m[1][0] * m[3][2] - m[1][2] * m[3][0])) * inv_det,

                (m[0][0] * (m[1][2] * m[2][3] - m[1][3] * m[2][2]) -
                m[0][2] * (m[1][0] * m[2][3] - m[1][3] * m[2][0]) +
                m[0][3] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])) * inv_det,
            ],
            // row 2
            [
                (m[1][0] * (m[2][1] * m[3][3] - m[2][3] * m[3][1]) -
                m[1][1] * (m[2][0] * m[3][3] - m[2][3] * m[3][0]) +
                m[1][3] * (m[2][0] * m[3][1] - m[2][1] * m[3][0])) * inv_det,

                (m[0][1] * (m[2][0] * m[3][3] - m[2][3] * m[3][0]) -
                m[0][0] * (m[2][1] * m[3][3] - m[2][3] * m[3][1]) -
                m[0][3] * (m[2][0] * m[3][1] - m[2][1] * m[3][0])) * inv_det,

                (m[0][0] * (m[1][1] * m[3][3] - m[1][3] * m[3][1]) -
                m[0][1] * (m[1][0] * m[3][3] - m[1][3] * m[3][0]) +
                m[0][3] * (m[1][0] * m[3][1] - m[1][1] * m[3][0])) * inv_det,

                (m[0][1] * (m[1][0] * m[2][3] - m[1][3] * m[2][0]) -
                m[0][0] * (m[1][1] * m[2][3] - m[1][3] * m[2][1]) -
                m[0][3] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])) * inv_det,
            ],
            // row 3
            [
                (m[1][1] * (m[2][3] * m[3][0] - m[2][0] * m[3][3]) +
                m[1][2] * (m[2][0] * m[3][1] - m[2][1] * m[3][0]) +
                m[1][0] * (m[2][1] * m[3][2] - m[2][2] * m[3][1])) * inv_det,

                (m[0][0] * (m[2][1] * m[3][2] - m[2][2] * m[3][1]) -
                m[0][1] * (m[2][0] * m[3][2] - m[2][2] * m[3][0]) +
                m[0][2] * (m[2][0] * m[3][1] - m[2][1] * m[3][0])) * inv_det,

                (m[0][1] * (m[1][3] * m[3][0] - m[1][0] * m[3][3]) +
                m[0][2] * (m[1][0] * m[3][1] - m[1][1] * m[3][0]) +
                m[0][0] * (m[1][1] * m[3][2] - m[1][2] * m[3][1])) * inv_det,

                (m[0][0] * (m[1][2] * m[2][1] - m[1][1] * m[2][2]) +
                m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0]) +
                m[0][2] * (m[1][1] * m[2][0] - m[1][0] * m[2][1])) * inv_det,
            ],
        ];

        Ok(Matrix::new(inv))
    }
}

impl Matrix<3, 3>
{

    pub fn rotation_x(angle_rad: f32) -> Matrix<3, 3> {
        let cos = angle_rad.cos();
        let sin = angle_rad.sin();
        
        Matrix::new([
            [  1.0,  0.0,  0.0],
            [  0.0,  cos, -sin],
            [  0.0,  sin,  cos]
        ])
    }
    
    pub fn rotation_y(angle_rad: f32) -> Matrix<3, 3> {
        let cos = angle_rad.cos();
        let sin = angle_rad.sin();
        
        Matrix::new([
            [  cos,  0.0,  sin],
            [  0.0,  1.0,  0.0],
            [ -sin,  0.0,  cos],
        ])
    }
    
    pub fn rotation_z(angle_rad: f32) -> Matrix<3, 3> {
        let cos = angle_rad.cos();
        let sin = angle_rad.sin();
        
        Matrix::new([
            [  cos,  -sin,  0.0],
            [  sin,  cos,  0.0],
            [  0.0,  0.0,  1.0],
        ])
    }

    pub fn rotation_axis(axis: VulkanVector<3>, angle_rad: f32) -> Result<Matrix<3, 3>, &'static str> {

        let norm = axis.normalize()?;  // опасно там ERR

        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();
        let cos_v = 1.0 - cos_a;

        // Элементы матрицы по формуле Rodrigues
        // https://mathworld.wolfram.com/RodriguesRotationFormula.html
        let xx = norm[0] * norm[0] * cos_v;
        let xy = norm[0] * norm[1] * cos_v;
        let xz = norm[0] * norm[2] * cos_v;
        let yy = norm[1] * norm[1] * cos_v;
        let yz = norm[1] * norm[2] * cos_v;
        let zz = norm[2] * norm[2] * cos_v;

        let kx_sin = norm[0] * sin_a;
        let ky_sin = norm[1] * sin_a;
        let kz_sin = norm[2] * sin_a;

        Ok(Matrix::new([
            [ xx + cos_a, xy - kz_sin, xz + ky_sin],
            [xy + kz_sin,  yy + cos_a, yz - kx_sin],
            [xz - ky_sin, yz + kx_sin,  zz + cos_a],
        ]))
        /*
            R = I + w*sin(th) + w^2 * (1 - cos(th))
            I - identity matrix
            w :
            [   0  -vz -vy ]
            [  vz    0 -vx ]
            [ -vy   vx   0 ]
            v - rotation vector 
        */
    }

    pub fn inverse(&self) -> Result<Self, &'static str> {
        let a = self[0][0]; let b = self[0][1]; let c = self[0][2];
        let d = self[1][0]; let e = self[1][1]; let f = self[1][2];
        let g = self[2][0]; let h = self[2][1]; let i = self[2][2];

        let det = a * (e * i - f * h)
                - b * (d * i - f * g)
                + c * (d * h - e * g);

        if det.abs() < 1e-10 {
            return Err("Matrix is singular (determinant is zero)");
        }

        let inv_det = 1.0 / det;

        Ok(Matrix::new([
            [
                (e * i - f * h) * inv_det,
                (c * h - b * i) * inv_det,
                (b * f - c * e) * inv_det,
            ],
            [
                (f * g - d * i) * inv_det,
                (a * i - c * g) * inv_det,
                (c * d - a * f) * inv_det,
            ],
            [
                (d * h - e * g) * inv_det,
                (b * g - a * h) * inv_det,
                (a * e - b * d) * inv_det,
            ],
        ]))
    }
}

impl<const ROWS: usize, const COLS: usize> 
    Add for Matrix<ROWS, COLS> {
    type Output = Matrix<ROWS, COLS>;
    
    fn add(self, other: Self) -> Self::Output {
        let mut result = self.clone();
        for i in 0..ROWS {
            for j in 0..COLS {
                result[i][j] = result[i][j] + other[i][j];
            }
        }
        result
    }
}

impl<const ROWS: usize, const COLS: usize> 
    Sub for Matrix<ROWS, COLS> {
    type Output = Matrix<ROWS, COLS>;
    
    fn sub(self, other: Self) -> Self::Output {
        let mut result = self.clone();
        for i in 0..ROWS {
            for j in 0..COLS {
                result[i][j] = result[i][j] - other[i][j];
            }
        }
        result
    }
}

impl<const ROWS: usize, const COLS: usize, const COMMON: usize> 
    Mul<Matrix<COMMON, COLS>> for Matrix<ROWS, COMMON> {
    type Output = Matrix<ROWS, COLS>;
    
    fn mul(self, other: Matrix<COMMON, COLS>) -> Self::Output {
        let mut result_data = [[0.0; COLS]; ROWS];
        
        for i in 0..ROWS {
            for j in 0..COLS {
                let mut sum = 0.0;
                for k in 0..COMMON {
                    sum = sum + self[i][k] * other[k][j];
                }
                result_data[i][j] = sum;
            }
        }
        
        Matrix::new(result_data)
    }
    
}

#[derive(Copy, Clone)]
struct Scalar(f32);  // orphan rule круто

impl<const ROWS: usize, const COLS: usize> 
    Mul<Scalar> for Matrix<ROWS, COLS> {
    type Output = Matrix<ROWS, COLS>;
    
    fn mul(self, scalar: Scalar) -> Self::Output {
        let mut result = self.clone();
        for i in 0..ROWS {
            for j in 0..COLS {
                result[i][j] = result[i][j] * scalar.0;
            }
        }
        result
    }
}

impl<const ROWS: usize, const COLS: usize> 
    Mul<Matrix<ROWS, COLS>> for Scalar {
    type Output = Matrix<ROWS, COLS>;
    
    fn mul(self, matrix: Matrix<ROWS, COLS>) -> Self::Output {
        matrix * self
    }
}

impl<const ROWS: usize, const COLS: usize> fmt::Display 
    for Matrix<ROWS, COLS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..ROWS {
            write!(f, "[")?;
            for j in 0..COLS {
                write!(f, "{}", self[i][j])?;
                if j < COLS - 1 {
                    write!(f, ", ")?;
                }
            }
            writeln!(f, "]")?;
        }
        Ok(())
    }
}


impl<const ROWS: usize, const COLS: usize> 
    Mul<VulkanVector<COLS>> for Matrix<ROWS, COLS> {
    type Output = VulkanVector<ROWS>;
    
    fn mul(self, other: VulkanVector<COLS>) -> Self::Output {
        let mut result_data = [0.0; ROWS];
        
        for i in 0..ROWS {
            for j in 0..COLS {
                result_data[i] += self[i][j] * other[j];
            }
        }
        
        VulkanVector::new(result_data)
    }
}


impl<const ROWS: usize, const COLS: usize> Index<usize> for Matrix<ROWS, COLS> {
    type Output = [f32; COLS];
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<const ROWS: usize, const COLS: usize> IndexMut<usize> for Matrix<ROWS, COLS> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]   
    }
}

