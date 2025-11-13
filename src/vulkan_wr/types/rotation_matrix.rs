// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: матрицы вращения. реализованы через трейт для матриц (думал может оно понадобятся в векторах и в матрицах)
// есть матрицы 2, 3, 4д, хотя вращение возможно лишь по xyz
// есть еще Rodrigues формула для произвольной оси-вектора, не реализовано
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use super::matrix::Matrix;
use std::ops::{Sub, Mul, Add};

pub trait RotationMatrix<T, const DIM: usize> {

    fn rotation_x(angle_rad: f32) -> Matrix<T, DIM, DIM>;
    fn rotation_y(angle_rad: f32) -> Matrix<T, DIM, DIM>;
    fn rotation_z(angle_rad: f32) -> Matrix<T, DIM, DIM>;
    
    // Вспомогательные методы для градусов
    fn rotation_x_deg(angle_deg: f32) -> Matrix<T, DIM, DIM> {
        Self::rotation_x(angle_deg.to_radians())
    }
    fn rotation_y_deg(angle_deg: f32) -> Matrix<T, DIM, DIM> {
        Self::rotation_y(angle_deg.to_radians())
    }
    fn rotation_z_deg(angle_deg: f32) -> Matrix<T, DIM, DIM> {
        Self::rotation_z(angle_deg.to_radians())
    }
}

// Реализация для 2x2 матриц (2D вращение)
impl<T> RotationMatrix<T, 2> for Matrix<T, 2, 2>
where
    T: Default + Copy + From<f32> + Mul<Output = T> + Add<Output = T> + Sub<Output = T>,
{   
    fn rotation_x(_angle_rad: f32) -> Matrix<T, 2, 2> {
        // Для 2D нет оси X, возвращаем единичную матрицу
        Matrix::identity()
    }
    
    fn rotation_y(_angle_rad: f32) -> Matrix<T, 2, 2> {
        // Для 2D нет оси Y, возвращаем единичную матрицу
        Matrix::identity()
    }
    
    fn rotation_z(angle_rad: f32) -> Matrix<T, 2, 2> {
        // Для 2D ось Z перпендикулярна плоскости, используем rotation_2d
        let cos = angle_rad.cos();
        let sin = angle_rad.sin();
        
        Matrix::new([
            [T::from(cos), T::from(-sin)],
            [T::from(sin), T::from(cos)],
        ])
    }
}

// Реализация для 3x3 матриц (3D вращение)
impl<T> RotationMatrix<T, 3> for Matrix<T, 3, 3>
where
    T: Default + Copy + From<f32> + Mul<Output = T> + Add<Output = T> + Sub<Output = T>,
{
    fn rotation_x(angle_rad: f32) -> Matrix<T, 3, 3> {
        let cos = angle_rad.cos();
        let sin = angle_rad.sin();
        
        Matrix::new([
            [T::from(1.0), T::from(0.0), T::from(0.0)],
            [T::from(0.0), T::from(cos), T::from(-sin)],
            [T::from(0.0), T::from(sin), T::from(cos)],
        ])
    }
    
    fn rotation_y(angle_rad: f32) -> Matrix<T, 3, 3> {
        let cos = angle_rad.cos();
        let sin = angle_rad.sin();
        
        Matrix::new([
            [T::from(cos),  T::from(0.0), T::from(sin)],
            [T::from(0.0),  T::from(1.0), T::from(0.0)],
            [T::from(-sin), T::from(0.0), T::from(cos)],
        ])
    }
    
    fn rotation_z(angle_rad: f32) -> Matrix<T, 3, 3> {
        let cos = angle_rad.cos();
        let sin = angle_rad.sin();
        
        Matrix::new([
            [T::from(cos), T::from(-sin), T::from(0.0)],
            [T::from(sin), T::from(cos),  T::from(0.0)],
            [T::from(0.0), T::from(0.0),  T::from(1.0)],
        ])
    }
}

// Реализация для 4x4 матриц (3D вращение с однородными координатами)
impl<T> RotationMatrix<T, 4> for Matrix<T, 4, 4>
where
    T: Default + Copy + From<f32> + Mul<Output = T> + Add<Output = T> + Sub<Output = T>,
{
    fn rotation_x(angle_rad: f32) -> Matrix<T, 4, 4> {
        let cos = angle_rad.cos();
        let sin = angle_rad.sin();
        
        Matrix::new([
            [T::from(1.0), T::from(0.0), T::from(0.0), T::from(0.0)],
            [T::from(0.0), T::from(cos), T::from(-sin), T::from(0.0)],
            [T::from(0.0), T::from(sin), T::from(cos),  T::from(0.0)],
            [T::from(0.0), T::from(0.0), T::from(0.0),  T::from(1.0)],
        ])
    }
    
    fn rotation_y(angle_rad: f32) -> Matrix<T, 4, 4> {
        let cos = angle_rad.cos();
        let sin = angle_rad.sin();
        
        Matrix::new([
            [T::from(cos),  T::from(0.0), T::from(sin),  T::from(0.0)],
            [T::from(0.0),  T::from(1.0), T::from(0.0),  T::from(0.0)],
            [T::from(-sin), T::from(0.0), T::from(cos),  T::from(0.0)],
            [T::from(0.0),  T::from(0.0), T::from(0.0),  T::from(1.0)],
        ])
    }
    
    fn rotation_z(angle_rad: f32) -> Matrix<T, 4, 4> {
        let cos = angle_rad.cos();
        let sin = angle_rad.sin();
        
        Matrix::new([
            [T::from(cos), T::from(-sin), T::from(0.0), T::from(0.0)],
            [T::from(sin), T::from(cos),  T::from(0.0), T::from(0.0)],
            [T::from(0.0), T::from(0.0),  T::from(1.0), T::from(0.0)],
            [T::from(0.0), T::from(0.0),  T::from(0.0), T::from(1.0)],
        ])
    }
}
