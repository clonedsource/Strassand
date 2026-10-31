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

fn NaiveMultiplication(matr_a: &Matrix, matr_b: &Matrix) -> Matrix {
    let mut matr_c: Matrix = Matrix{dim: (matr_a.dim.0, matr_b.dim.1), data: Vec::new()};
    // c_ij = a_i1*b1_j + a_i2+b2_j + ... + a_in*b_nj 
    for m in 1..matr_c.dim.0 + 1 {
        for n in 1..matr_c.dim.1 + 1 {
            matr_c.data.push(0_f64);
            let mn =matr_c.index(m,n);
            for s in 1..matr_a.dim.1 + 1 {
                matr_c.data[mn] += matr_a.data[matr_a.index(m,s)]*matr_b.data[matr_b.index(s,n)]
            }
            
        }
    }
    matr_c
}

fn StrassandMultiplication(matr_a: &Matrix, matr_b: &Matrix) -> Matrix {
    // Make sure A & B are properly sized (2^n) x (2^n).
    
    // Begin dividing them into (2x2) chunks recursively.

    // Once at (2x2) Chunks use 7 M Matrices.
    
    // Combine.
    let mut matr_c: Matrix = Matrix{dim: (matr_a.dim.0, matr_b.dim.1), data: Vec::new()};
    // c_ij = a_i1*b1_j + a_i2+b2_j + ... + a_in*b_nj 
    let matr_d = NaiveMultiplication(&matr_a, &matr_b);
    matr_a.print_size();
    matr_c
}
fn main() {
    let a: Matrix = Matrix{dim: (3,3), data: vec![1_f64, 4_f64, 5_f64, 2_f64, 3_f64, 7_f64,0_f64,0_f64,0_f64]};
    let b: Matrix = Matrix{dim: (3,3), data: vec![0_f64, 1_f64, 0_f64, 4_f64, 0_f64, 0_f64, 5_f64, 3_f64, 0_f64]};
    let c: Matrix = NaiveMultiplication(&a, &b);

    let d: Matrix = Matrix{dim: (2,3), data: vec![1_f64, 4_f64, 5_f64, 2_f64, 3_f64, 7_f64]};
    let e: Matrix = Matrix{dim: (3,2), data: vec![0_f64, 1_f64, 4_f64, 0_f64, 5_f64, 3_f64]};
    let f: Matrix = NaiveMultiplication(&d, &e);
    //let g: Matrix = StrassandMultiplication(&d, &e);
    //c.print();
    //f.print();
    //c.print_size();
    println!("MEMORY USED: {} BYTES", a.size() + b.size() + c.size());
}
