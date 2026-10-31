use std::{mem::size_of, ops::{Add, AddAssign, Index}, vec};

#[derive(Debug, PartialEq, Clone)]
struct Matrix {
    pub dim: (usize, usize),
    pub data: Vec<f64>,
}

//Canonical Size of Matrix is 2 usizes (4 bytes ea.) [Total 8 Bytes], 1 Vec (32 bytes ea.), Total Byte Size is 40.

impl Add<Matrix> for Matrix {
    type Output = Self;
    fn add(self, rhs: Matrix) -> Self::Output {
        let mut matrix_new: Matrix = Matrix{dim: self.dim.clone(), data: Vec::new()};
        for i in 0_usize .. (self.dim.0*self.dim.1) {
            matrix_new.data.push(self.data.index(i) + rhs.data.index(i))
        }
        matrix_new
    }
}
impl AddAssign<Matrix> for Matrix {
    fn add_assign(&mut self, rhs: Matrix) {
         for i in 0_usize .. (self.dim.0*self.dim.1) {
            self.data[i] += rhs.data.index(i)
        }
    }
}

impl Matrix {
    fn print(&self) {
        println!("Dim(M) = ({},{})",self.dim.0, self.dim.1);
        let mut matrix_string: String = String::new();
        for row in 0 .. (self.dim.0) {
            matrix_string.push_str("[");
            for col_entry_idx in 0 .. (self.dim.1) {
                let entry: &f64 = self.data.index((row * self.dim.1) + col_entry_idx);
                matrix_string.push_str(&entry.to_string());
                if col_entry_idx == self.dim.1 - 1 {continue};
                matrix_string.push_str(" ");
            }
            matrix_string.push_str("]\n");
        }
        print!("{}",matrix_string);
    }
    fn size(&self) -> usize {
        println!("{}, {}", std::mem::size_of::<Matrix>(),std::mem::size_of_val(&*self.data));
        std::mem::size_of_val(&*self.data) + std::mem::size_of::<Matrix>()
    }
    fn print_size(&self) {
        let size_data_bytes: usize = std::mem::size_of_val(&*self.data);
        let size_total_bytes: usize = std::mem::size_of::<Matrix>()+size_data_bytes;
        let data_fmt: &str = match (size_data_bytes as f64).log(1000_f64) as u8 {
            3 => "GB",
            2 => "MB",
            1 => "KB",
            0 => "B",
            _ => "?B",
        };
        let total_fmt: &str = match (size_total_bytes as f64).log(1000_f64) as u8 {
            3 => "GB",
            2 => "MB",
            1 => "KB",
            0 => "B",
            _ => "?B",
        };
         ((size_data_bytes as f64) / (1000_f64.powi((size_data_bytes as f64).log(1000_f64).floor() as i32))) as usize;
        println!("Header -> {} B\nData -> {} {}\n    Total = {} {}",
            std::mem::size_of::<Matrix>(),
            ((size_data_bytes as f64)/ (1000_f64.powi((size_data_bytes as f64).log(1000_f64).floor() as i32))) as usize,
            data_fmt,
            ((size_total_bytes as f64)/ (1000_f64.powi((size_total_bytes as f64).log(1000_f64).floor() as i32))) as usize,
            total_fmt,
        );
    }
    fn index(&self, x: usize, y: usize) -> usize {
        ((x - 1) * self.dim.1) + (y - 1)
    }
}