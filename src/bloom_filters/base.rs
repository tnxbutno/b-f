use std::f64::consts::LN_2;

pub trait Filter {
    fn new(n: u32, f: f64) -> Self;
    fn insert(&mut self, value: &[u8]);
    fn lookup(&self, value: &[u8]) -> bool;
    fn get_size(&self) -> usize;

    /// m = -(nlε/(ln2)^2) where ε is desired false positive probability,
    /// in our case it is indicated by the letter f
    fn calculate_m(f: f64, n: u32) -> u64 {
        -(f.ln() * n as f64 / (LN_2.powi(2))).ceil() as u64
    }

    /// k = m/n * ln2
    fn calculate_k(m: u64, n: u32) -> u64 {
        ((m / n as u64) as f64 * LN_2).ceil() as u64
    }
}
