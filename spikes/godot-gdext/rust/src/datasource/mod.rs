pub trait DataSource: Send + Sync {
    fn has(&self, path: &str) -> bool;
    fn read(&self, path: &str) -> Option<Vec<u8>>;
}

pub mod compound;
pub mod mpq;

pub use compound::CompoundDataSource;
pub use mpq::MpqDataSource;
