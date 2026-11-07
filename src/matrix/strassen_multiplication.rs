use super::*;
const CUTOFF: usize = 32;
fn padder(matr: &Matrix<f64>, max_matr_dim: usize) -> Matrix<f64> {
    let log_matr_dim: usize = ((max_matr_dim as f64).log2().ceil() as u64) as usize;
    let padded_matr_dim: usize = 2 << log_matr_dim - 1;
    
    let mut padded_matr: Matrix<f64> = Matrix::new(
        (padded_matr_dim, padded_matr_dim), 
        Some(vec![0_f64;padded_matr_dim << log_matr_dim])
    );
    for m in 1 .. matr.dim.0 + 1 {
        for n in 1 .. matr.dim.1 + 1 {
            let padded_idx: usize = padded_matr.index(m, n);
            let idx: usize = matr.index(m,n);
            padded_matr.data[padded_idx] = matr.data[idx];
        }
    }
    padded_matr
}
fn quadrants(matr: &Matrix<f64>) -> (Matrix<f64>, Matrix<f64>, Matrix<f64>, Matrix<f64>) {
    let sub_size: usize = matr.dim.0 >> 1;
    let mut a11: Matrix<f64> = Matrix::new((sub_size, sub_size), Some(vec![0_f64;sub_size * sub_size]));
    let mut a12: Matrix<f64> = a11.clone();
    let mut a21: Matrix<f64> = a11.clone();
    let mut a22: Matrix<f64> = a11.clone();
    for m in 1 .. (sub_size << 1) + 1 {
        for n in 1 .. (sub_size << 1) + 1 {
            let idx: usize = matr.index(m,n);
            if m <= sub_size && n <= sub_size {
                let a11_idx: usize = a11.index(m,n);
                a11.data[a11_idx] = matr.data[idx];
            } else if m <= sub_size && n > sub_size {
                let a12_idx: usize = a12.index(m,n - sub_size);
                a12.data[a12_idx] = matr.data[idx];
            } else if m > sub_size && n <= sub_size {
                let a21_idx: usize = a21.index(m - sub_size,n);
                a21.data[a21_idx] = matr.data[idx];
            } else if m > sub_size && n > sub_size {
                let a22_idx: usize = a22.index(m - sub_size,n - sub_size);
                a22.data[a22_idx] = matr.data[idx];
            }
        }
    }
    (a11, a12, a21, a22)
}
fn join(matr_a11: Matrix<f64>, matr_a12: Matrix<f64>, matr_a21: Matrix<f64>, matr_a22: Matrix<f64>) -> Matrix<f64> {
    let sub_size: usize = matr_a11.dim.0;
    let mut joined_matr: Matrix<f64> = Matrix::new((sub_size << 1, sub_size << 1), Some(vec![0_f64;(sub_size << 1) * (sub_size << 1)]));
    for m in 1 .. (sub_size << 1) + 1 {
        for n in 1 .. (sub_size << 1) + 1 {
            let idx: usize = joined_matr.index(m,n);
            if m <= sub_size && n <= sub_size {
                let a11_idx: usize = matr_a11.index(m,n);
                joined_matr.data[idx] = matr_a11.data[a11_idx];
            } else if m <= sub_size && n > sub_size {
                let a12_idx: usize = matr_a12.index(m,n - sub_size);
                joined_matr.data[idx] = matr_a12.data[a12_idx];
            } else if m > sub_size && n <= sub_size {
                let a21_idx: usize = matr_a21.index(m - sub_size,n);
                joined_matr.data[idx] = matr_a21.data[a21_idx];
            } else if m > sub_size && n > sub_size {
                let a22_idx: usize = matr_a22.index(m - sub_size,n - sub_size);
                joined_matr.data[idx] = matr_a22.data[a22_idx];
            }
        }
    }
    joined_matr
}
fn strassen_recurse(matr_a: &Matrix<f64>, matr_b: &Matrix<f64>) -> (Matrix<f64>, usize) {
    // Goes back to traditional matrix multiplication when the matrix is small enough.
    if matr_a.dim.0 <= CUTOFF {
        let naive = naive_method(&matr_a, &matr_b);
        return (naive.0.unwrap(), naive.1);
    }
    let (a11, a12, a21, a22) = quadrants(&matr_a);
    let (b11, b12, b21, b22) = quadrants(&matr_b);
    
    let mut multiplications: usize = 0;
    let mut strassen_recurse_macro = |a: &Matrix<f64>, b: &Matrix<f64>|-> Matrix<f64> {
        let (matr, multi) = strassen_recurse(a, b);
        multiplications += multi;
        return matr;
    };
    let m1: Matrix<f64> = strassen_recurse_macro(&(&a11 + &a22), &(&b11 + &b22));
    let m2: Matrix<f64> = strassen_recurse_macro(&(&a21 + &a22), &b11);
    let m3: Matrix<f64> = strassen_recurse_macro(&a11, &(&b12 - &b22));
    let m4: Matrix<f64> = strassen_recurse_macro(&a22, &(&b21 - &b11));
    let m5: Matrix<f64> = strassen_recurse_macro(&(&a11 + &a12), &b22);
    let m6: Matrix<f64> = strassen_recurse_macro(&(&a21 - &a11), &(&b11 + &b12));
    let m7: Matrix<f64> = strassen_recurse_macro(&(&a12 - &a22), &(&b21 + &b22));
    (join(
        &(&m1 + &m4) + &(&m7 - &m5),
        &m3 + &m5,
        &m2 + &m4,
        &(&m1 - &m2) + &(&m3 + &m6)
    ), multiplications)
}
pub fn strassen_method(matr_a: &Matrix<f64>, matr_b: &Matrix<f64>) -> (Result<Matrix<f64>, &'static str>, usize) {
    if matr_a.dim.1 != matr_b.dim.0 {return (Err("Cannot Multiply: (mxn)(qxr) where n != q."), 0)}
    if matr_a.dim.0 <= CUTOFF {
        return naive_method(&matr_a, &matr_b);
    }
    // Padded Matrix A & B.
    let (padded_matr_a, padded_matr_b) = {
         // Find Maximum Dimension.
        let max_matr_dim: usize = {
            let max_matr_a_dim: usize = matr_a.dim.0.max(matr_a.dim.1);
            let max_matr_b_dim: usize = matr_b.dim.0.max(matr_b.dim.1);
            max_matr_a_dim.max(max_matr_b_dim)
        };  
       
        (padder(matr_a, max_matr_dim), padder(matr_b, max_matr_dim))
    };
    // Call Strassen Recursive Function.
    let (padded_result_matr_c, multiplications) = strassen_recurse(&padded_matr_a, &padded_matr_b);
    //padded_result_matr_c.print((true, true));
    // Generate Appropriately sized matrix.
    let mut matr_c: Matrix<f64> = Matrix::new((matr_a.dim.0, matr_b.dim.1), Some(vec![0_f64;matr_a.dim.0*matr_b.dim.1]));
    //matr_c.print((true, true));
    (Ok(matr_c), multiplications)

}