#![allow(non_snake_case, dead_code, unused_mut, unused_variables)]
mod matrix;
use matrix::*;
use std::{time::Instant};


fn NaiveMultiplication(matr_a: &Matrix<f64>, matr_b: &Matrix<f64>) -> Result<Matrix<f64>, &'static str> {
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

fn StrassandMultiplication(matr_a: &Matrix<f64>, matr_b: &Matrix<f64>) -> Result<Matrix<f64>, &'static str> {
    if matr_a.dim.1 != matr_b.dim.0 {return Err("Cannot Multiply: (mxn)(qxr) where n != q.")}

    // Padding Lambda Function.
    let padder = |matr: &Matrix<f64>, max_matr_dim: usize| -> Matrix<f64> {
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
    };
   
    // Padded Matrix A & B.
    let (mut padded_matr_a, mut padded_matr_b) = {
         // Find Maximum Dimension.
        let max_matr_dim: usize = {
            let max_matr_a_dim: usize = matr_a.dim.0.max(matr_a.dim.1);
            let max_matr_b_dim: usize = matr_b.dim.0.max(matr_b.dim.1);
            max_matr_a_dim.max(max_matr_b_dim)
        };
        (padder(matr_a, max_matr_dim), padder(matr_b, max_matr_dim))
    };

    // Generate Appropriately sized matrix.
    let mut matr_c: Matrix<f64> = Matrix::new((matr_a.dim.0, matr_b.dim.1), Some(vec![0_f64;matr_a.dim.0*matr_b.dim.1]));
    
    // Partition matr_a and matr_b into 2x2 pointers. Create M1-M7. Do Math. Recombine. Do Math. Recombine.
    // Hints: Take log2(matr_a)
    let mut partition_vec: Vec<(usize, usize)> = Vec::new();
   
    // Accurately partitions 
    let mut depth: usize = (padded_matr_a.dim.0 as f64).log2() as usize;
    while depth > 1 {
        depth -= 1;
        println!("{}, {}x{}", 1 << (depth << 1), padded_matr_a.dim.0 >> depth,  padded_matr_a.dim.0 >> depth); // 2n, 1/2
        //todo()! make partition vectors for a & b, then do math. then combine.
    }

    matr_c.print((true, true));
    Ok(matr_c)

}
fn REF64(matrix: &mut Matrix<f64>) {
    matrix.print((false,false));
    println!("+++++ REF +++++");
    let now = Instant::now();
    for n in 1 .. matrix.dim.1 {
        let ri_idx: usize = matrix.index(n,n);
        if matrix.data[ri_idx] == 0_f64 {continue};
        for ri in n .. matrix.dim.0 {
            let rj_idx: usize = matrix.index(ri + 1,n);
            let factor: f64 = -matrix.data[rj_idx] / matrix.data[ri_idx];
            matrix.ero3(n, ri + 1, factor);
            //matrix.print((false,false));
        }
    }
    println!("+++++ FINISHED ({:?}) +++++", now.elapsed());
    matrix.print((false,false));

}

fn main() {
    let (mut b, mut c) = (
        random_matrix::<f64>((2,3)),
        random_matrix::<f64>((3,2))
    );
    identity_matrix64(3).print((true,true));
    diagonal_matrix64(vec![4_f64, 5_f64, 2_f64, 10_f64], None).print((true,true));
    //n.print((true, true));
    //n1.print((true, true));
    //b.print((false,false));
    //println!("x");
    //c.print((false,false));
    //println!("=");
    //println!("+++++ NAIVE MATR MULTI +++++");
    //let mut now = Instant::now();
    //match NaiveMultiplication(&b, &c) {
    //    Ok(matr) => matr.print((false,false)),
    //    Err(res) => println!("{}", res),
    //};
    //println!("+++++ FINISHED ({:?}) +++++", now.elapsed());

    //println!("+++++ STRASSAND MATR MULTI +++++");
    //now = Instant::now();
    //match StrassandMultiplication(&b, &c) {
    //    Ok(matr) => matr.print((false,false)),
    //    Err(res) => println!("{}", res),
    //};
    //println!("+++++ FINISHED ({:?}) +++++", now.elapsed());
    println!("===== MATRIX USED MEMORY =====");

}
