use crate::matrix::{ERO, MatrixConstructor, MatrixFormatters};
use std::{ops::{Add, AddAssign, Index, Mul, MulAssign, Sub, SubAssign}, vec};

#[derive(Debug, PartialEq, Clone)]
pub struct Matrix64 {
    pub dim: (usize, usize),
    pub data: Vec<f64>,
}

//Canonical Size of Matrix64 is 2 usizes (4 bytes ea.) [Total 8 Bytes], 1 Vec (32 bytes ea.), Total Byte Size is 40.
impl MatrixConstructor<f64> for Matrix64 {
    type Output = Self;
    fn new(dim: (usize, usize), data: Option<Vec<f64>>) -> Self::Output {
        return Self::Output{
            dim: dim,
            data: match data {
                Some(field) => field,
                None => vec![0_f64;dim.0*dim.1],
            }
        }
    }
}

impl MatrixFormatters for Matrix64 {
    fn size_str(&self) -> String {
        let size_bytes: usize = self.size();
        // GB_COUNT, FMT_STR
        return format!("{} {}",
            ((size_bytes as f64) / (1000_f64.powi((size_bytes as f64).log(1000_f64).floor() as i32))),
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
                let entry: &f64 = self.data.index((row * self.dim.1) + col_entry_idx);
                
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
impl Matrix64 {
    // Accessors
    pub fn size(&self) -> usize {
        // Heap (Vector) + Stack (Header) Size.
        std::mem::size_of_val(&*self.data) + std::mem::size_of::<Self>()
    }
    
    pub fn index(&self, x: usize, y: usize) -> usize {((x - 1) * self.dim.1) + (y - 1)}

    pub fn print(&self, print_opts: (bool, bool)) {
        // Print Options: (dimension, size[MEM])
        println!("{}", self.matrix_str());
        if print_opts.0 {println!("Dim(M): {}", self.dim_str())};
        if print_opts.1 {println!("Size(M): {}", self.size_str())};
    }
}
impl ERO<f64> for Matrix64 {
    fn ero1(&mut self, ri: usize, rj: usize) { // ri <=> rj
        for n in 1 .. self.dim.1 + 1 {
            let (rin, rjn) = (self.index(ri, n), self.index(rj, n));
            self.data.swap(rin,rjn);
        }
    }

    fn ero2(&mut self, ri: usize, ci: f64) { // ri* = ci*ri
        for n in 1 .. self.dim.1 + 1 {
            let idx = self.index(ri, n);
            self.data[idx] *= ci;
        }
    }

    fn ero3(&mut self, ri: usize, rj: usize, k: f64) { // rj* = rj + k*ri
        for n in 1 .. self.dim.1 + 1 {
            let (rin, rjn) = (self.index(ri, n), self.index(rj, n));
            self.data[rjn] += self.data[rin] * k;
        }
    }
}
impl Add<&Matrix64> for &Matrix64 {
    type Output = Matrix64;
    fn add(self, rhs: &Matrix64) -> Self::Output {
        let mut matrix_new: Self::Output = Self::Output::new(self.dim.clone(), Some(Vec::new()));
        for i in 0_usize .. (self.dim.0*self.dim.1) {
            matrix_new.data.push(self.data.index(i) + rhs.data.index(i))
        }
        matrix_new
    }
}
impl Add<Matrix64> for Matrix64 {
    type Output = Matrix64;
    fn add(self, rhs: Matrix64) -> Self::Output {
        let mut matrix_new: Self::Output = Self::Output::new(self.dim.clone(), Some(Vec::new()));
        for i in 0_usize .. (self.dim.0*self.dim.1) {
            matrix_new.data.push(self.data.index(i) + rhs.data.index(i))
        }
        matrix_new
    }
}

impl AddAssign<&Matrix64> for Matrix64 {
    fn add_assign(&mut self, rhs: &Matrix64) {
         for i in 0_usize .. (self.dim.0*self.dim.1) {
            self.data[i] += rhs.data.index(i)
        }
    }
}
impl AddAssign<Matrix64> for Matrix64 {
    fn add_assign(&mut self, rhs: Matrix64) {
         for i in 0_usize .. (self.dim.0*self.dim.1) {
            self.data[i] += rhs.data.index(i)
        }
    }
}

impl Sub<&Matrix64> for &Matrix64 {
    type Output = Matrix64;
    fn sub(self, rhs: &Matrix64) -> Self::Output {
        let mut matrix_new: Self::Output = Self::Output::new(self.dim.clone(), Some(Vec::new()));
        for i in 0_usize .. (self.dim.0*self.dim.1) {
            matrix_new.data.push(self.data.index(i) - rhs.data.index(i))
        }
        matrix_new
    }
}
impl Sub<Matrix64> for Matrix64 {
    type Output = Matrix64;
    fn sub(self, rhs: Matrix64) -> Self::Output {
        let mut matrix_new: Self::Output = Self::Output::new(self.dim.clone(), Some(Vec::new()));
        for i in 0_usize .. (self.dim.0*self.dim.1) {
            matrix_new.data.push(self.data.index(i) - rhs.data.index(i))
        }
        matrix_new
    }
}

impl SubAssign<&Matrix64> for Matrix64 {
    fn sub_assign(&mut self, rhs: &Matrix64) {
         for i in 0_usize .. (self.dim.0*self.dim.1) {
            self.data[i] -= rhs.data.index(i)
        }
    }
}
impl SubAssign<Matrix64> for Matrix64 {
    fn sub_assign(&mut self, rhs: Matrix64) {
         for i in 0_usize .. (self.dim.0*self.dim.1) {
            self.data[i] -= rhs.data.index(i)
        }
    }
}

impl Mul<f64> for &Matrix64 {
    type Output = Matrix64;
    fn mul(self, rhs: f64) -> Self::Output {
        let mul_data: Vec<f64> = self.data.clone()
        .iter()
        .map(|entry| entry * rhs)
        .collect::<Vec<_>>();
        Self::Output::new(self.dim.clone(), Some(mul_data))
    }
}
impl Mul<f64> for Matrix64 {
    type Output = Matrix64;
    fn mul(self, rhs: f64) -> Self::Output {
        let mul_data: Vec<f64> = self.data.clone()
        .iter()
        .map(|entry| entry * rhs)
        .collect::<Vec<_>>();
        Self::Output::new(self.dim.clone(), Some(mul_data))
    }
}
impl MulAssign<f64> for Matrix64 {
    fn mul_assign(&mut self, rhs: f64) {
        for entry in self.data.iter_mut() {
            *entry *= rhs
        }
    }
}
impl Mul<f32> for &Matrix64 {
    type Output = Matrix64;
    fn mul(self, rhs: f32) -> Self::Output {
        let mul_data: Vec<f64> = self.data.clone()
        .iter()
        .map(|entry| entry * rhs as f64)
        .collect::<Vec<_>>();
        Self::Output::new(self.dim.clone(), Some(mul_data))
    }
}
impl Mul<f32> for Matrix64 {
    type Output = Matrix64;
    fn mul(self, rhs: f32) -> Self::Output {
        let mul_data: Vec<f64> = self.data.clone()
        .iter()
        .map(|entry| entry * rhs as f64)
        .collect::<Vec<_>>();
        Self::Output::new(self.dim.clone(), Some(mul_data))
    }
}
impl MulAssign<f32> for Matrix64 {
    fn mul_assign(&mut self, rhs: f32) {
        for entry in self.data.iter_mut() {
            *entry *= rhs as f64
        }
    }
}

pub fn diagonal_matrix64(entries: Vec<f64>, size: Option<usize>) -> Matrix64 {
    // Finds which is smaller (if there is a size), the number of entries? or the size?
    let end: usize = match size {
        Some(v) => {v.min(entries.len())},
        None => entries.len(),
    };
    // Creates the allocated memory.
    let mut matrix_entries: Vec<f64> = vec![0_f64;end*end];
    // Lambda (anon) func. to turn a R^2 -> R^1.
    let index = |x: usize, y: usize| {((x - 1) * end) + (y - 1)};
    // Creates Diagonal Entries.
    for i in 1 .. end + 1 {
        matrix_entries[index(i,i)] = entries[i - 1];
    }
    // The pie is done baking.
    Matrix64::new((end, end), Some(matrix_entries))
}
pub fn identity_matrix64(size: usize) -> Matrix64 {diagonal_matrix64(vec![1_f64;size], None)}