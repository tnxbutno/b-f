use std::f64::consts::LN_2;
use bit_vec::BitVec;
use xxhash_rust::xxh3::{xxh3_64_with_seed};

pub struct BloomFilter {
    // number of bits in a Bloom filter
    m: u64,
    // number of hash functions
    k: u64,

    storage: BitVec,
}

impl BloomFilter {
    // n -- number of elements to insert
    // f -- the false positive rate
    pub fn new(n: u32, f: f64) -> Self {
        let m = Self::calculate_m(f, n);
        Self {
            m,
            k: Self::calculate_k(m, n),
            storage: BitVec::from_elem(m as usize, false),
        }
    }

    // m = -(nlε/(ln2)^2) where ε is desired false positive probability,
    // in our case it is indicated by the letter f
    fn calculate_m(f: f64, n: u32) -> u64 {
        -(f.ln() * n as f64 / (LN_2.powi(2))).ceil() as u64
    }

    // k = m/n * ln2
    fn calculate_k(m: u64, n: u32) -> u64 {
        ((m / n as u64) as f64 * LN_2).ceil() as u64
    }

    pub fn insert(&mut self, value: &[u8]) {
        let hash1 = xxh3_64_with_seed(value, 0) % self.m;
        let hash2 = xxh3_64_with_seed(value, 64) % self.m;
        for i in 0..self.k {
            let idx = ((hash1 + i * hash2) % self.m) as usize;
            self.storage.set(idx, true);
        }
    }

    pub fn lookup(&self, value: &[u8]) -> bool {
        let hash1 = xxh3_64_with_seed(value, 0) % self.m;
        let hash2 = xxh3_64_with_seed(value, 64) % self.m;
        for i in 0..self.k {
            let idx = ((hash1 + i * hash2) % self.m) as usize;
            if self.storage.get(idx) == Some(false) {
                return false
            }
        }
        true
    }

    pub fn get_size(&self) -> usize {
        self.storage.len()
    }
}

pub struct PartitionedBloomFilter {
    // number of bits in a Bloom filter
    m: u64,
    // number of hash functions
    k: u64,

    partition_size: usize,
    partitions: Vec<BitVec>,
}

impl PartitionedBloomFilter {
    // n -- number of elements to insert
    // f -- the false positive rate
    pub fn new(n: u32, f: f64) -> Self {
        let m = Self::calculate_m(f, n);
        let k = Self::calculate_k(m, n);
        let partition_size = (m/k) as usize;
        Self {
            m,
            k,
            partition_size,
            partitions: std::iter::repeat(BitVec::from_elem(partition_size, false))
                .take(k as usize).collect()
        }
    }

    // m = -(nlε/(ln2)^2) where ε is desired false positive probability,
    // in our case it is indicated by the letter f
    fn calculate_m(f: f64, n: u32) -> u64 {
        -(f.ln() * n as f64 / (LN_2.powi(2))).ceil() as u64
    }

    // k = m/n * ln2
    fn calculate_k(m: u64, n: u32) -> u64 {
        ((m / n as u64) as f64 * LN_2).ceil() as u64
    }

    pub fn insert(&mut self, value: &[u8]) {
        let hash1 = xxh3_64_with_seed(value, 0) % self.partition_size as u64;
        let hash2 = xxh3_64_with_seed(value, 64) % self.partition_size as u64;
        for i in 0..self.k {
            let idx = ((hash1 + i * hash2) % self.partition_size as u64) as usize;
            self.partitions[i as usize].set(idx, true);
        }
    }

    pub fn lookup(&self, value: &[u8]) -> bool {
        let hash1 = xxh3_64_with_seed(value, 0) % self.partition_size as u64;
        let hash2 = xxh3_64_with_seed(value, 64) % self.partition_size as u64;
        for i in 0..self.k {
            let idx = ((hash1 + i * hash2) % self.partition_size as u64) as usize;
            if self.partitions[i as usize].get(idx) == Some(false) {
                return false
            }
        }
        true
    }

    pub fn get_size(&self) -> usize {
        self.partitions.len()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use super::*;
    use rand::distributions::{Uniform};
    use rand::{Rng, thread_rng};

    #[test]
    fn simple_check() {
        let mut bf = BloomFilter::new(10, 0.01);
        bf.insert(&1u32.to_be_bytes());
        bf.insert(&10u32.to_be_bytes());
        bf.insert(&30u32.to_be_bytes());

        let res = bf.lookup(&1u32.to_be_bytes());
        assert!(res, "stored value is not found!");

        let res = bf.lookup(&10u32.to_be_bytes());
        assert!(res, "stored value is not found!");

        let res = bf.lookup(&30u32.to_be_bytes());
        assert!(res, "stored value is not found!");

        let res = bf.lookup(&45u32.to_be_bytes());
        assert!(!res, "not stored value is found!");
    }

    #[test]
    fn partitioned_simple_check() {
        let mut bf = PartitionedBloomFilter::new(10, 0.01);
        bf.insert(&1u32.to_be_bytes());
        bf.insert(&10u32.to_be_bytes());
        bf.insert(&30u32.to_be_bytes());

        let res = bf.lookup(&1u32.to_be_bytes());
        assert!(res, "stored value is not found!");

        let res = bf.lookup(&10u32.to_be_bytes());
        assert!(res, "stored value is not found!");

        let res = bf.lookup(&30u32.to_be_bytes());
        assert!(res, "stored value is not found!");

        let res = bf.lookup(&45u32.to_be_bytes());
        assert!(!res, "not stored value is found!");
    }

    #[test]
    fn verify_false_positive_rate() {
        let mut bf = BloomFilter::new(10u32.pow(7), 0.02);
        let mut track_inserted = HashSet::new();

        let mut rng = thread_rng();
        let distribution = Uniform::new_inclusive(0, 10u64.pow(12));
        for _ in 0..10u32.pow(7) {
            let value = rng.sample(distribution).to_be_bytes();
            bf.insert(&value);
            track_inserted.insert(value);
        }

        let mut false_positive = 0;
        for _ in 0..10u32.pow(6) {
            let value = rng.sample(distribution).to_be_bytes();
            let found = bf.lookup(&value);
            if found && track_inserted.get(&value).is_none() {
                false_positive += 1;
            }
        }

        dbg!("regular", false_positive);
        // check that false positive rate is ~2%
        assert!(19900 < false_positive && false_positive < 21000);
    }

    #[test]
    fn verify_partitioned_bf_false_positive_rate() {
        let mut bf = PartitionedBloomFilter::new(10u32.pow(7), 0.02);
        let mut track_inserted = HashSet::new();

        let mut rng = thread_rng();
        let distribution = Uniform::new_inclusive(0, 10u64.pow(12));
        for _ in 0..10u32.pow(7) {
            let value = rng.sample(distribution).to_be_bytes();
            bf.insert(&value);
            track_inserted.insert(value);
        }

        let mut false_positive = 0;
        for _ in 0..10u32.pow(6) {
            let value = rng.sample(distribution).to_be_bytes();
            let found = bf.lookup(&value);
            if found && track_inserted.get(&value).is_none() {
                false_positive += 1;
            }
        }

        dbg!("partitioned", false_positive);
        // check that false positive rate is ~2%
        assert!(19900 < false_positive && false_positive < 21000);
    }
}
