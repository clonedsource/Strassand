#![allow(unused_imports)]
use std::{fmt::Display, ops::{Add, AddAssign, Index, Mul, MulAssign, Sub, SubAssign}};
use rand::prelude::*;

mod special_matrices;
mod naive_multiplication;
mod strassen_multiplication;
pub use {special_matrices::*, naive_multiplication::*, strassen_multiplication::*};
pub trait MatrixConstructor<Numeric = Self> {
    type Output;
    fn new(dim: (usize, usize), data: Option<Vec<Numeric>>) -> Self::Output;
}
pub trait MatrixFormatters {
    fn size_str(&self) -> String;
    fn dim_str(&self) -> String;
    fn matrix_str(&self) -> String;
}
pub trait ERO<T> {
    fn ero1(&mut self, ri: usize, rj: usize); // Swaps ri and rj
    fn ero2(&mut self, ri: usize, ci: T); // Multiplies ri by ci
    fn ero3(&mut self, ri: usize, rj: usize, k: T); // Adds k*rj to ri.
}

#[derive(Debug, PartialEq, Clone)]
pub struct Matrix<T>{
    pub dim: (usize, usize),
    pub data: Vec<T>,
}

//Canonical Size of Matrix is 2 usizes (4 bytes ea.) [Total 8 Bytes], 1 Vec (32 bytes ea.), Total Byte Size is 40.
impl<T: Display> Matrix<T> {
    // Accessors
    pub fn size(&self) -> usize {
        // Heap (Vector) + Stack (Header) Size.
        std::mem::size_of_val(&*self.data) + std::mem::size_of::<Self>()
    }

    pub fn print(&self, print_opts: (bool, bool)) {
        // Print Options: (dimension, size[MEM])
        println!("{}", self.matrix_str());
        if print_opts.0 {println!("Dim(M): {}", self.dim_str())};
        if print_opts.1 {println!("Size(M): {}", self.size_str())};
    }
    
}

impl<T: Clone + From<i8> + Into<T>> Matrix<T> {
    pub fn index(&self, x: usize, y: usize) -> usize {((x - 1) * self.dim.1) + (y - 1)}
    pub fn position(&self, idx: usize) -> (usize, usize) {((idx + 1 / self.dim.1), ((idx + 1) % self.dim.1) + 1)}
}

impl<T: Clone + From<i8> + Into<T>> MatrixConstructor<T> for Matrix<T> {
    type Output = Self;
    fn new(dim: (usize, usize), data: Option<Vec<T>>) -> Self::Output {
        return Self::Output{
            dim: dim,
            data: match data {
                Some(field) => field,
                None => vec![0.into();dim.0*dim.1],
            }
        }
    }
}

impl<T: Display> MatrixFormatters for Matrix<T> {
    fn size_str(&self) -> String {
        let size_bytes: usize = self.size();
        // GB_COUNT, FMT_STR
        return format!("{} {}",
            ((size_bytes as usize) / (1000_f64.powi((size_bytes as f64).log(1000_f64).floor() as i32)) as usize),
            match (size_bytes as f64).log(1000_f64) as u8 {
                3 => "GB", 2 => "MB", 1 => "KB", 0 => "B", _ => "?B",
            }
        );
    }
    fn dim_str(&self) -> String {return format!("({},{})",self.dim.0, self.dim.1)}
    fn matrix_str(&self) -> String {
        let mut matrix_string: String = String::new();
        for row in 0 .. (self.dim.0) {
            matrix_string.push_str("[");
            for col_entry_idx in 0 .. (self.dim.1) {
                let entry: &T = self.data.index((row * self.dim.1) + col_entry_idx);
                
                matrix_string.push_str(&entry.to_string());
                if col_entry_idx == self.dim.1 - 1 {continue};
                matrix_string.push_str(" ");
            }
            matrix_string.push_str("]");
            if row < self.dim.0 - 1 {matrix_string.push_str("\n");}
        }
        return matrix_string;
    }
}

impl<T> ERO<T> for Matrix<T> where 
    T: Copy + 
    Clone + 
    From<i8> + 
    Into<T> +
    MulAssign<T> +
    Mul<T, Output = T> +
    AddAssign<T>

{
    fn ero1(&mut self, ri: usize, rj: usize) { // ri <=> rj
        for n in 1 .. self.dim.1 + 1 {
            let (rin, rjn) = (self.index(ri, n), self.index(rj, n));
            self.data.swap(rin,rjn);
        }
    }

    fn ero2(&mut self, ri: usize, ci: T) { // ri* = ci*ri
        for n in 1 .. self.dim.1 + 1 {
            let idx = self.index(ri, n);
            self.data[idx] *= ci;
        }
    }

    fn ero3(&mut self, ri: usize, rj: usize, k: T) { // rj* = rj + k*ri
        for n in 1 .. self.dim.1 + 1 {
            let (rin, rjn) = (self.index(ri, n), self.index(rj, n));
            let entry = self.data[rin] * k;
            self.data[rjn] += entry;
        }
    }
}

impl<T> Add<&Matrix<T>> for &Matrix<T> where 
    T: Copy + 
    Clone + 
    From<i8> + 
    Into<T> +
    Add<T, Output = T>
{
    type Output = Matrix<T>;
    fn add(self, rhs: &Matrix<T>) -> Self::Output {
        let mut matrix_new: Self::Output = Matrix::new(self.dim.clone(), Some(Vec::new()));
        for i in 0_usize .. (self.dim.0*self.dim.1) {
            matrix_new.data.push(*self.data.index(i) + *rhs.data.index(i));
        }
        matrix_new
    }
}
impl<T> Add<Matrix<T>> for Matrix<T> where 
    T: Copy + 
    Clone + 
    From<i8> + 
    Into<T> +
    Add<T, Output = T>
{
    type Output = Matrix<T>;
    fn add(self, rhs: Matrix<T>) -> Self::Output {
        let mut matrix_new: Self::Output = Self::Output::new(self.dim.clone(), Some(Vec::new()));
        for i in 0_usize .. (self.dim.0*self.dim.1) {
            matrix_new.data.push(*self.data.index(i) + *rhs.data.index(i))
        }
        matrix_new
    }
}

impl<T> AddAssign<&Matrix<T>> for Matrix<T> where 
    T: Copy + 
    Clone + 
    From<i8> + 
    Into<T> +
    AddAssign<T>
{
    fn add_assign(&mut self, rhs: &Matrix<T>) {
         for i in 0_usize .. (self.dim.0*self.dim.1) {
            self.data[i] += *rhs.data.index(i)
        }
    }
}
impl<T> AddAssign<Matrix<T>> for Matrix<T> where 
    T: Copy + 
    Clone + 
    From<i8> + 
    Into<T> +
    AddAssign<T>
{
    fn add_assign(&mut self, rhs: Matrix<T>) {
         for i in 0_usize .. (self.dim.0*self.dim.1) {
            self.data[i] += *rhs.data.index(i)
        }
    }
}



impl<T> Sub<&Matrix<T>> for &Matrix<T> where 
    T: Copy + 
    Clone + 
    From<i8> + 
    Into<T> +
    Sub<T, Output = T>
{
    type Output = Matrix<T>;
    fn sub(self, rhs: &Matrix<T>) -> Self::Output {
        let mut matrix_new: Self::Output = Matrix::new(self.dim.clone(), Some(Vec::new()));
        for i in 0_usize .. (self.dim.0*self.dim.1) {
            matrix_new.data.push(*self.data.index(i) - *rhs.data.index(i));
        }
        matrix_new
    }
}
impl<T> Sub<Matrix<T>> for Matrix<T> where 
    T: Copy + 
    Clone + 
    From<i8> + 
    Into<T> +
    Sub<T, Output = T>
{
    type Output = Matrix<T>;
    fn sub(self, rhs: Matrix<T>) -> Self::Output {
        let mut matrix_new: Self::Output = Self::Output::new(self.dim.clone(), Some(Vec::new()));
        for i in 0_usize .. (self.dim.0*self.dim.1) {
            matrix_new.data.push(*self.data.index(i) - *rhs.data.index(i))
        }
        matrix_new
    }
}

impl<T> SubAssign<&Matrix<T>> for Matrix<T> where 
    T: Copy + 
    Clone + 
    From<i8> + 
    Into<T> +
    SubAssign<T>
{
    fn sub_assign(&mut self, rhs: &Matrix<T>) {
         for i in 0_usize .. (self.dim.0*self.dim.1) {
            self.data[i] -= *rhs.data.index(i)
        }
    }
}
impl<T> SubAssign<Matrix<T>> for Matrix<T> where 
    T: Copy + 
    Clone + 
    From<i8> + 
    Into<T> +
    SubAssign<T>
{
    fn sub_assign(&mut self, rhs: Matrix<T>) {
         for i in 0_usize .. (self.dim.0*self.dim.1) {
            self.data[i] -= *rhs.data.index(i)
        }
    }
}



impl<T> Mul<T> for &Matrix<T> where 
    T: Copy +
    Clone +
    From<i8> +
    Mul<T, Output = T>
{
    type Output = Matrix<T>;
    fn mul(self, rhs: T) -> Self::Output {
        let mul_data: Vec<T> = self.data.clone()
        .iter()
        .map(|entry| (*entry) * rhs as T)
        .collect::<Vec<_>>();
        Self::Output::new(self.dim.clone(), Some(mul_data))
    }
}
impl<T> Mul<T> for Matrix<T> where 
    T: Copy +
    Clone +
    From<i8> +
    Mul<T, Output = T>
{
    type Output = Matrix<T>;
    fn mul(self, rhs: T) -> Self::Output {
        let mul_data: Vec<T> = self.data.clone()
        .iter()
        .map(|entry| (*entry) * rhs as T)
        .collect::<Vec<_>>();
        Self::Output::new(self.dim.clone(), Some(mul_data))
    }
}
impl<T> MulAssign<T> for Matrix<T> where 
    T: Copy +
    Clone +
    From<i8> +
    MulAssign<T>
{
    fn mul_assign(&mut self, rhs: T) {
        for entry in self.data.iter_mut() {
            *entry *= rhs as T;
        }
    }
}

pub fn compute_matrix<F, T>(dim: (usize, usize), mut closure: F) -> Matrix<T> where
    F: FnMut(usize, usize) -> T,
    T: Copy + Clone + From<i8> + Into<T>
{

    let mut random_vec: Vec<T> = vec![0.into();dim.0*dim.1];
    for i in 0 .. random_vec.len() {
        let row: usize = i + 1 / dim.1;
        let col: usize = ((i + 1) % dim.1) + 1;
        random_vec[i] = closure(row, col);
    }
    Matrix::new(dim, Some(random_vec))
}

pub fn random_matrix<T>(dim: (usize, usize)) -> Matrix<T> where
    T: Copy + Clone + From<i8> + From<i16> + Into<T>
{
    let mut rng = rand::rng();
    let closure = |_x: usize, _y: usize| {rng.random_range(i8::MIN+1_i8..i8::MAX).into()};
    compute_matrix(dim, closure)
}