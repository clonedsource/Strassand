#![allow(non_snake_case, dead_code,unused_mut)]
mod matrix;
use matrix::*;
use std::{fmt::Error, time::Instant};
use rand::{prelude::*, seq::index};
fn NaiveMultiplication(matr_a: &Matrix64, matr_b: &Matrix64) -> Result<Matrix64, &'static str> {
    if matr_a.dim.1 != matr_b.dim.0 {return Err("Cannot Multiply: (mxn)(qxr) where n != q.")}
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
    Ok(matr_c)
}

fn StrassandMultiplication(matr_a: &Matrix64, matr_b: &Matrix64) -> Result<Matrix64, &'static str> {
    if matr_a.dim.1 != matr_b.dim.0 {return Err("Cannot Multiply: (mxn)(qxr) where n != q.")}

    // Padding Lambda Function.
    let padder = |matr: &Matrix64, max_matr_dim: usize| -> Matrix64 {
 
        let log_matr_dim: usize = ((max_matr_dim as f64).log2().ceil() as u64) as usize;
        let padded_matr_dim: usize = 2 << log_matr_dim - 1;
        
        let mut padded_matr: Matrix64 = Matrix64::new((padded_matr_dim, padded_matr_dim), Some(vec![0_f64;padded_matr_dim << log_matr_dim]));
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
    let mut matr_c: Matrix64 = Matrix64::new((matr_a.dim.0, matr_b.dim.1), Some(vec![0_f64;matr_a.dim.0*matr_b.dim.1]));
    
    // Partition matr_a and matr_b into 2x2 pointers. Create M1-M7. Do Math. Recombine. Do Math. Recombine.
    // Hints: Take log2(matr_a)

    matr_c.print((true, true));
    Ok(matr_c)
    
}
fn REF64(matrix: &mut Matrix64) {
    matrix.print((false,false));
    println!("+++++ REF +++++");
    let now = Instant::now();
    for n in 1 .. matrix.dim.1 {
        let ri_idx: usize = matrix.index(n,n);
        if matrix.data[ri_idx] == 0_f64 {continue};
        for ri in n .. matrix.dim.0 {
            let rj_idx: usize = matrix.index(ri + 1,n);
            let factor = -matrix.data[rj_idx] / matrix.data[ri_idx];
            matrix.ero3(n, ri + 1, factor);
            //matrix.print((false,false));
        }
    }
    println!("+++++ FINISHED ({:?}) +++++", now.elapsed());
    matrix.print((false,false));

}

fn generate_random_mn_matr64(dim: (usize, usize)) -> Matrix64 {
    let mut random_vec: Vec<f64> = vec![0_f64;dim.0*dim.1];
    let mut rng = rand::rng();
    for i in 0 .. random_vec.len() {random_vec[i] = rng.random_range(-20_i64..20_i64) as f64;}
    Matrix64::new(dim, Some(random_vec))
}

fn main() {
    let (mut b, mut c) = (generate_random_mn_matr64((5,3)),generate_random_mn_matr64((3,17)));
    
    
    //b.print((false,false));
    println!("x");
    //c.print((false,false));
    println!("=");

    println!("+++++ NAIVE MATR MULTI +++++");
    let mut now = Instant::now();
    match NaiveMultiplication(&b, &c) {
        Ok(matr) => matr.print((false,false)),
        Err(res) => println!("{}", res),
    };
    println!("+++++ FINISHED ({:?}) +++++", now.elapsed());

    println!("+++++ STRASSAND MATR MULTI +++++");
    now = Instant::now();
    match StrassandMultiplication(&b, &c) {
        Ok(matr) => matr.print((false,false)),
        Err(res) => println!("{}", res),
    };
    println!("+++++ FINISHED ({:?}) +++++", now.elapsed());

    //REF64(&mut b.clone());
    println!("===== MATRIX USED MEMORY =====");

}
