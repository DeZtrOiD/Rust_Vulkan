// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: Небольшая обертка для векторов
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#

use std::{f32::EPSILON, fmt, ops::{Add, AddAssign, Mul, Sub, SubAssign, Index, IndexMut}};

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct VulkanVector<const SIZE: usize> {
    pub data: [f32; SIZE],
}

impl<const SIZE: usize> VulkanVector<SIZE> {
    pub fn new(data: [f32; SIZE]) -> Self {
        VulkanVector { data }
    }
}

impl<const SIZE: usize> VulkanVector<SIZE> {
    pub fn normalize(&self) -> Result<Self, &'static str> {
        let norm = (self.data.iter().map(|x| (*x) * (*x)).sum::<f32>()).sqrt();
        if norm < EPSILON * 10.0 {
            return Err("Division by 0. Norm is zero.");
        }
        let data_norm = std::array::from_fn(|i| self[i] / norm);
        Ok(Self { data: data_norm })
    }
    pub fn dot(&self, other: &Self) -> f32 {
        self.data.iter().enumerate().map(|(i, val)| val * other[i]).sum()
    }
}

impl VulkanVector<3> {
    pub fn cross(&self, other: &Self) -> Self {
        VulkanVector::new([
            self[1] * other[2] - self[2] * other[1],
            self[2] * other[0] - self[0] * other[2],
            self[0] * other[1] - self[1] * other[0],
        ])
    }
    pub fn from4(vec: VulkanVector<4>) -> Self {
        VulkanVector::new([vec[0], vec[1], vec[2]])
    }
    pub fn to4v(&self, f: f32) -> VulkanVector<4> {
        VulkanVector::new([self[0], self[1], self[2], f])
    }
}
impl<const SIZE: usize> From<[f32; SIZE]> for VulkanVector<SIZE> {
    fn from(value: [f32; SIZE]) -> Self {
        Self { data: value }
    }
}

impl<const SIZE: usize> Into<[f32; SIZE]> for VulkanVector<SIZE> {
    fn into(self) -> [f32; SIZE] {
        self.data
    }
}

impl VulkanVector<4> {
    pub fn from3(vec: VulkanVector<3>, f: f32) -> Self {
        VulkanVector::new([vec[0], vec[1], vec[2], f])
    }
    pub fn to3v(&self) -> VulkanVector<3> {
        if self.data[3].abs() > EPSILON * 100.0 {
            let inv = 1.0 / self.data[3];
            VulkanVector { data: [self.data[0] * inv, self.data[1] * inv, self.data[2] * inv] }
        } else {
            VulkanVector { data: [self.data[0], self.data[1], self.data[2]] }
        }
    }
}

impl<const SIZE: usize> fmt::Display for VulkanVector<SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..SIZE {
            write!(f, "[")?;
            write!(f, "{}", self[i])?;
            writeln!(f, "]")?;
        }
        Ok(())
    }
}

impl<const SIZE: usize> AddAssign for VulkanVector<SIZE> {
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..SIZE {
            self[i] += rhs[i];
        }
    }
}

impl<const SIZE: usize> Mul<f32> for VulkanVector<SIZE> {
    type Output = VulkanVector<SIZE>;
    fn mul(self, rhs: f32) -> Self::Output {
        let mut data = self.data;
        for i in 0..SIZE {
            data[i] = data[i] * rhs; 
        }

        Self { data: data }
    }
}

impl<const SIZE: usize> Add for VulkanVector<SIZE> {
    type Output = VulkanVector<SIZE>;
    fn add(self, rhs: Self) -> Self::Output {
        let data: [f32; SIZE] = std::array::from_fn(|i| self[i] + rhs[i]);
        Self{data: data}
    }
}

impl<const SIZE: usize> Sub for VulkanVector<SIZE> {
    type Output = VulkanVector<SIZE>;
    fn sub(self, rhs: Self) -> Self::Output {
        let data: [f32; SIZE] = std::array::from_fn(|i| self[i] - rhs[i]);
        Self{data: data}
    }
}

impl<const SIZE: usize>  SubAssign for VulkanVector<SIZE> {
    fn sub_assign(&mut self, rhs: Self) {
        for i in 0..SIZE {
            self[i] -= rhs[i];
        }
    }
}

impl<const SIZE: usize> Default for VulkanVector<SIZE> { fn default() -> Self { VulkanVector { data: [0.0; SIZE] } } }

impl<const SIZE: usize> Index<usize> for VulkanVector<SIZE> {
    type Output = f32;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<const SIZE: usize> IndexMut<usize> for VulkanVector<SIZE> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]   
    }
}
