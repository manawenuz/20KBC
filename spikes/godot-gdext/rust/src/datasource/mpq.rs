use std::path::Path;
use std::sync::Mutex;

use wow_mpq::Archive;

use super::DataSource;

/// Wraps a `wow-mpq` archive handle and implements the `DataSource` trait.
///
/// Path normalization (forward → back slashes) and case-insensitive lookup
/// are handled natively by `wow-mpq`, so no extra conversion is required.
pub struct MpqDataSource {
    archive: Mutex<Archive>,
}

impl MpqDataSource {
    pub fn open(path: &Path) -> Result<Self, String> {
        Archive::open(path)
            .map(|archive| Self {
                archive: Mutex::new(archive),
            })
            .map_err(|e| e.to_string())
    }
}

impl DataSource for MpqDataSource {
    fn has(&self, path: &str) -> bool {
        let archive = self.archive.lock().unwrap();
        matches!(archive.find_file(path), Ok(Some(_)))
    }

    fn read(&self, path: &str) -> Option<Vec<u8>> {
        let mut archive = self.archive.lock().unwrap();
        archive.read_file(path).ok()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    #[cfg(target_os = "macos")]
    fn mpq_reads_peasant_mdx() {
        let mpq_path = Path::new("/Volumes/samGames/WC3/War3.mpq");
        if !mpq_path.exists() {
            return;
        }

        let mpq = MpqDataSource::open(mpq_path).unwrap();
        assert!(mpq.has("Units/Human/Peasant/peasant.mdx"));

        let bytes = mpq.read("Units/Human/Peasant/peasant.mdx").unwrap();
        assert!(bytes.starts_with(b"MDLX"));
        assert!(bytes.len() > 100_000);
    }
}
