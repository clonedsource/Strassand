pub trait MatrixConstructor<Numeric = Self> {
    type Output;
    fn new(dim: (usize, usize), data: Option<Vec<Numeric>>) -> Self::Output;
}
pub trait MatrixFormatters {
    fn size_str(&self) -> String;
    fn dim_str(&self) -> String;
    fn matrix_str(&self) -> String;
}

pub trait ERO<T> {
    fn ero1(&mut self, ri: usize, rj: usize); // Swaps ri and rj
    fn ero2(&mut self, ri: usize, ci: T); // Multiplies ri by ci
    fn ero3(&mut self, ri: usize, rj: usize, k: T); // Adds k*rj to ri.
}
//f128 and f16 are nightly builds. no access.
//mod matrix128;
mod matrix64;
mod matrix32;

//pub use matrix128::*;
pub use matrix64::*;
pub use matrix32::*;