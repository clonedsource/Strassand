use super::*;
use std::{alloc::{Layout, alloc, dealloc}, ptr, sync::mpsc::*, thread::{self, JoinHandle}};
pub fn naive_method(matr_a: &Matrix<f64>, matr_b: &Matrix<f64>) -> (Result<Matrix<f64>, &'static str>, usize) {
    if matr_a.dim.1 != matr_b.dim.0 {return (Err("Cannot Multiply: (mxn)(qxr) where n != q."), 0)}
    let mut matr_c: Matrix<f64> = Matrix::new((matr_a.dim.0, matr_b.dim.1), Some(vec![0_f64;matr_a.dim.0 * matr_b.dim.1]));
    let mut multiplications: usize = 0;
    // c_ij = a_i1*b1_j + a_i2+b2_j + ... + a_in*b_nj
    for m in 1..matr_c.dim.0 + 1 {
        for n in 1..matr_c.dim.1 + 1 {
            let mn =matr_c.index(m,n);
            for s in 1..matr_a.dim.1 + 1 {
                matr_c.data[mn] += matr_a.data[matr_a.index(m,s)] * matr_b.data[matr_b.index(s,n)];
                multiplications += 1;
            }
            
        }
    }
    (Ok(matr_c), multiplications)
}