#![allow(non_snake_case, dead_code,unused_mut)]
mod matrix;
use matrix::*;

fn NaiveMultiplication(matr_a: &Matrix64, matr_b: &Matrix64) -> Matrix64 {
    let mut matr_c: Matrix64 = Matrix64::new((matr_a.dim.0, matr_b.dim.1), Some(Vec::new()));
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

fn StrassandMultiplication(matr_a: &Matrix64, matr_b: &Matrix64) -> Matrix64 {
    // Make sure A & B are properly sized (2^n) x (2^n).
    
    // Begin dividing them into (2x2) chunks recursively.

    // Once at (2x2) Chunks use 7 M Matrices.
    
    // Combine.
    let mut matr_c: Matrix64 = Matrix64::new((matr_a.dim.0, matr_b.dim.1), Some(Vec::new()));
    // c_ij = a_i1*b1_j + a_i2+b2_j + ... + a_in*b_nj 
    let matr_d = NaiveMultiplication(&matr_a, &matr_b);
    println!("Size(D): {}", matr_d.size_str());
    matr_c
}
fn REF64(matrix: &mut Matrix64) {
    matrix.print((false,false));
    println!("+++++ REF +++++");
    for n in 1 .. matrix.dim.1 {
        for ri in n .. matrix.dim.0 {
            let (ri_idx, rj_idx) = (matrix.index(n,n),matrix.index(ri + 1,n));
            let factor = -matrix.data[rj_idx] / matrix.data[ri_idx];
            matrix.ero3(n, ri + 1, factor);
            matrix.print((false,false));
        }
    }
    println!("+++++ FINISHED +++++");
    matrix.print((false,false));

}
fn main() {
    let mut a: Matrix64 = Matrix64::new((3,3), Some(vec![2_f64,1_f64,5_f64,4_f64,6_f64,7_f64,6_f64,11_f64,18_f64]));
    REF64(&mut a);
    println!("===== MATRIX USED MEMORY =====");
    a.print((false, true));
}
