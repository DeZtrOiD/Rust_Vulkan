// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: Небольшая обертка для матриц
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#

use std::{fmt, ops::{Add, Mul, Sub}, process::Output};

#[derive(Debug, Clone, PartialEq)]
pub struct Matrix<T, const ROWS: usize, const COLS: usize> {
    pub data: [[T; COLS]; ROWS]
}

impl<T: Default + Copy, const ROWS: usize, const COLS: usize> Matrix<T, ROWS, COLS> {
    pub fn new(data: [[T; COLS]; ROWS]) -> Self {
        Matrix { data }
    }

    pub fn rows(&self) -> usize {
        ROWS
    }

    pub fn cols(&self) -> usize {
        COLS
    }

    pub fn get(&self, row: usize, col: usize) -> Option<&T> {
        if row < ROWS && col < COLS {
            Some(&self.data[row][col])
        } else {
            None
        }
    }

    pub fn transpose(&self) -> Matrix<T, COLS, ROWS> {
        let mut result_data = [[self.data[0][0]; ROWS]; COLS];
        for i in 0..ROWS {
            for j in 0..COLS {
                result_data[j][i] = self.data[i][j];
            }
        }
        Matrix::new(result_data)
    }
    
    pub fn rotate(&self, rotation_matrix: &Matrix<T, ROWS, ROWS>) -> Result<Matrix<T, ROWS, COLS>, &'static str>
        where T: Default + Copy + From<f32> + Mul<Output = T> + Add<Output = T>
    {
        if ROWS != COLS {
            return Err("Rotation can only be applied to square matrices");
        }
        
        Ok(rotation_matrix.clone() * self.clone())
    }
}

impl<T: Default + Copy + From<f32>, const SIZE: usize> Matrix<T, SIZE, SIZE> {
    pub fn identity() -> Self {
        let mut data = [[T::default(); SIZE]; SIZE];
        for i in 0..SIZE {
            data[i][i] = T::from(1.0);
        }
        Matrix::new(data)
    }
}

impl<T: Add<Output = T> + Copy, const ROWS: usize, const COLS: usize> 
    Add for Matrix<T, ROWS, COLS> {
    type Output = Matrix<T, ROWS, COLS>;
    
    fn add(self, other: Self) -> Self::Output {
        let mut result = self.clone();
        for i in 0..ROWS {
            for j in 0..COLS {
                result.data[i][j] = result.data[i][j] + other.data[i][j];
            }
        }
        result
    }
}


impl<T: Sub<Output = T> + Copy, const ROWS: usize, const COLS: usize> 
    Sub for Matrix<T, ROWS, COLS> {
    type Output = Matrix<T, ROWS, COLS>;
    
    fn sub(self, other: Self) -> Self::Output {
        let mut result = self.clone();
        for i in 0..ROWS {
            for j in 0..COLS {
                result.data[i][j] = result.data[i][j] - other.data[i][j];
            }
        }
        result
    }
}

impl<T: Mul<Output = T> + Add<Output = T> + Copy + Default, const ROWS: usize, const COLS: usize, const COMMON: usize> 
    Mul<Matrix<T, COMMON, COLS>> for Matrix<T, ROWS, COMMON> {
    type Output = Matrix<T, ROWS, COLS>;
    
    fn mul(self, other: Matrix<T, COMMON, COLS>) -> Self::Output {
        let mut result_data = [[T::default(); COLS]; ROWS];
        
        for i in 0..ROWS {
            for j in 0..COLS {
                let mut sum = T::default();
                for k in 0..COMMON {
                    sum = sum + self.data[i][k] * other.data[k][j];
                }
                result_data[i][j] = sum;
            }
        }
        
        Matrix::new(result_data)
    }
}

#[derive(Copy, Clone)]
struct Scalar<T>(T);  // orphan rule круто

impl<T: Mul<Output = T> + Copy, const ROWS: usize, const COLS: usize> 
    Mul<Scalar<T>> for Matrix<T, ROWS, COLS> {
    type Output = Matrix<T, ROWS, COLS>;
    
    fn mul(self, scalar: Scalar<T>) -> Self::Output {
        let mut result = self.clone();
        for i in 0..ROWS {
            for j in 0..COLS {
                result.data[i][j] = result.data[i][j] * scalar.0;
            }
        }
        result
    }
}

impl<T: Mul<Output = T> + Copy, const ROWS: usize, const COLS: usize> 
    Mul<Matrix<T, ROWS, COLS>> for Scalar<T> {
    type Output = Matrix<T, ROWS, COLS>;
    
    fn mul(self, matrix: Matrix<T, ROWS, COLS>) -> Self::Output {
        matrix * self
    }
}

impl<T: fmt::Display, const ROWS: usize, const COLS: usize> fmt::Display 
    for Matrix<T, ROWS, COLS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..ROWS {
            write!(f, "[")?;
            for j in 0..COLS {
                write!(f, "{}", self.data[i][j])?;
                if j < COLS - 1 {
                    write!(f, ", ")?;
                }
            }
            writeln!(f, "]")?;
        }
        Ok(())
    }
}
