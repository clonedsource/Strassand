#![allow(non_snake_case, dead_code, unused_mut, unused_variables)]
mod matrix;
use matrix::*;
mod terminal;
use terminal::*;

use std::{time::{Instant}};
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
            matrix.print((false,false));
            println!("=========");
        }
    }
    println!("+++++ FINISHED ({:?}) +++++", now.elapsed());
    matrix.print((false,false));

}

fn main() {
    //env::args().for_each(|arg| println!("{}", arg));
    //terminal::begin();
    let (mut b, mut c) = (
        random_matrix::<f64>((5000,5000)),
        random_matrix::<f64>((5000,5000))
    );
    
    println!("A(size: {}, dim: {})", b.size_str(), b.dim_str());
    println!("x");
    println!("B(size: {}, dim: {})", c.size_str(), c.dim_str());
    println!("=");
    println!("+++++ NAIVE MATR MULTI +++++");
    let (mut now, mut _elapsed) = (Instant::now(), std::time::Duration::new(0,0));
    let (computed_naive, multiplications1) = naive_method(&b, &c);
    _elapsed = now.elapsed();
    //match computed_naive {
    //    Ok(matr) => matr.print((false,false)),
    //    Err(res) => println!("{}", res),
    //};
    println!("+++++ FINISHED ({:?}, {} multiplications) +++++", _elapsed, multiplications1);
    println!("+++++ STRASSEN MATR MULTI +++++");
    now = Instant::now();
    let (computed_strassen, multiplications2) = strassen_method(&b, &c);
    _elapsed = now.elapsed();
    //match computed_strassen {
    //    Ok(matr) => matr.print((false,false)),
    //    Err(res) => println!("{}", res),
    //};
    println!("+++++ FINISHED ({:?}, {} multiplications) +++++", _elapsed, multiplications2);
    println!("===== MATRIX USED MEMORY =====");
    println!("{}",b.size_str());
    println!("{}",c.size_str());
    println!("{}",&computed_strassen.unwrap().size_str());
}
