use super::DataSource;

/// Layered data source that queries a stack of sources in order (first match wins).
pub struct CompoundDataSource {
    sources: Vec<Box<dyn DataSource>>,
}

impl CompoundDataSource {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    pub fn add(&mut self, source: Box<dyn DataSource>) {
        self.sources.push(source);
    }
}

impl DataSource for CompoundDataSource {
    fn has(&self, path: &str) -> bool {
        self.sources.iter().any(|s| s.has(path))
    }

    fn read(&self, path: &str) -> Option<Vec<u8>> {
        self.sources.iter().find_map(|s| s.read(path))
    }
}
