// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: Небольшая обертка для векторов
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#

use std::{ops::{Add, Mul, Sub}, fmt};
use super::matrix::Matrix;


#[derive(Debug, Clone, PartialEq)]
pub struct Vector<T, const SIZE: usize> {
    pub data: [T; SIZE],
}

impl<T, const SIZE: usize> Vector<T, SIZE> {
    pub fn new(data: [T; SIZE]) -> Self {
        Vector { data }
    }
}


impl<T> Vector<T, 4>
where
    T: Mul<Output = T> + Add<Output = T> + Copy,
{    
    fn dot(&self, other: &Self) -> T {
        self.data[0] * other.data[0] + 
        self.data[1] * other.data[1] + 
        self.data[2] * other.data[2] +
        self.data[3] * other.data[3]
    }
}

impl<T> Vector<T, 3>
where
    T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Copy,
{    
    fn dot(&self, other: &Self) -> T {
        self.data[0] * other.data[0] + 
        self.data[1] * other.data[1] + 
        self.data[2] * other.data[2]
    }
    
    fn cross(&self, other: &Self) -> Self {
        Vector::new([
            self.data[1] * other.data[2] - self.data[2] * other.data[1],
            self.data[2] * other.data[0] - self.data[0] * other.data[2],
            self.data[0] * other.data[1] - self.data[1] * other.data[0],
        ])
    }
}

impl<T> Vector<T, 2>
where
    T: Mul<Output = T> + Add<Output = T> + Copy,
{    
    fn dot(&self, other: &Self) -> T {
        self.data[0] * other.data[0] + self.data[1] * other.data[1]
    }
}

impl<T, const SIZE: usize> Vector<T, SIZE> 
where
    T: Default + Copy + From<f32> + Mul<Output = T> + Add<Output = T> + Sub<Output = T>,
{
    pub fn rotate(&self, rotation_matrix: &Matrix<T, SIZE, SIZE>) -> Vector<T, SIZE> {
        // Конвертируем вектор в матрицу-столбец, вращаем и конвертируем обратно
        let vec_matrix = Matrix::new([self.data]).transpose();
        let rotated_matrix = rotation_matrix.clone() * vec_matrix;
        
        let mut result_data = [T::default(); SIZE];
        for i in 0..SIZE {
            result_data[i] = rotated_matrix.data[i][0];
        }
        
        Vector::new(result_data)
    }
}

impl<T: fmt::Display, const SIZE: usize> fmt::Display 
    for Vector<T, SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..SIZE {
            write!(f, "[")?;
            write!(f, "{}", self.data[i])?;
            writeln!(f, "]")?;
        }
        Ok(())
    }
}
