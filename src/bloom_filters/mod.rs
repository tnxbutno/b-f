mod base;
mod classical_bloom_filter;
mod partitioned_bloom_filter;

pub use self::base::Filter;
pub use self::classical_bloom_filter::ClassicalBloomFilter;
pub use self::partitioned_bloom_filter::PartitionedBloomFilter;
