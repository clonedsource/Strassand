use super::*;

pub fn naive_method(matr_a: &Matrix<f64>, matr_b: &Matrix<f64>) -> Result<Matrix<f64>, &'static str> {
    if matr_a.dim.1 != matr_b.dim.0 {return Err("Cannot Multiply: (mxn)(qxr) where n != q.")}
    let mut matr_c: Matrix<f64> = Matrix::new((matr_a.dim.0, matr_b.dim.1), Some(Vec::new()));
    // c_ij = a_i1*b1_j + a_i2+b2_j + ... + a_in*b_nj 
    for m in 1..matr_c.dim.0 + 1 {
        for n in 1..matr_c.dim.1 + 1 {
            matr_c.data.push(0_f64);
            let mn =matr_c.index(m,n);
            for s in 1..matr_a.dim.1 + 1 {
                matr_c.data[mn] += matr_a.data[matr_a.index(m,s)] * matr_b.data[matr_b.index(s,n)]
            }
            
        }
    }
    Ok(matr_c)
}