#![allow(non_snake_case, dead_code, unused_mut, unused_variables)]
mod matrix;
use matrix::*;
use std::{env, thread::sleep, time::{Instant}};
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
    env::args().for_each(|arg| println!("{}", arg));

    let (mut b, mut c) = (
        random_matrix::<f64>((5,3)),
        random_matrix::<f64>((3,5))
    );
    b.print((false,false));
    println!("x");
    c.print((false,false));
    println!("=");
    println!("+++++ NAIVE MATR MULTI +++++");
    let (mut now, mut elapsed) = (Instant::now(), std::time::Duration::new(0,0));
    let computed_naive = naive_method(&b, &c);
    elapsed = now.elapsed();
    match computed_naive {
        Ok(matr) => matr.print((false,false)),
        Err(res) => println!("{}", res),
    };
    println!("+++++ FINISHED ({:?}) +++++", elapsed);

    println!("+++++ STRASSAND MATR MULTI +++++");
    now = Instant::now();
    let computed_strassand = strassand_method(&b, &c);
    elapsed = now.elapsed();
    match computed_strassand {
        Ok(matr) => matr.print((false,false)),
        Err(res) => println!("{}", res),
    };
    println!("+++++ FINISHED ({:?}) +++++", elapsed);

    println!("===== MATRIX USED MEMORY =====");

}
