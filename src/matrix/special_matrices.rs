use super::*;
pub fn diagonal_matrix64(entries: Vec<f64>, size: Option<usize>) -> Matrix<f64> {
    // Finds which is smaller (if there is a size), the number of entries? or the size?
    let end: usize = match size {
        Some(v) => {v.min(entries.len())},
        None => entries.len(),
    };
    // Creates the allocated memory.
    let mut matrix_entries: Vec<f64> = vec![0_f64;end*end];
    // Lambda (anon) func. to turn a R^2 -> R^1.
    let index = |x: usize, y: usize| {((x - 1) * end) + (y - 1)};
    // Creates Diagonal Entries.
    for i in 1 .. end + 1 {
        matrix_entries[index(i,i)] = entries[i - 1];
    }
    // The pie is done baking.
    Matrix::new((end, end), Some(matrix_entries))
}
pub fn identity_matrix64(size: usize) -> Matrix<f64> {diagonal_matrix64(vec![1_f64;size], None)}