use super::*;

pub fn strassand_method(matr_a: &Matrix<f64>, matr_b: &Matrix<f64>) -> Result<Matrix<f64>, &'static str> {
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
    let mut depth: usize = (padded_matr_a.dim.0 as f64).log2() as usize + 1;
    while depth > 0 {
        depth -= 1;
        let magic_exponent: usize = 1 << (depth << 1);
        let partition_dim: usize = padded_matr_a.dim.0 >> depth;
        println!("{}, {}x{}", magic_exponent, partition_dim, partition_dim); // 2n, 1/2
        //todo()! make partition vectors for a & b, then do math. then combine.
    }

    matr_c.print((true, true));
    Ok(matr_c)

}