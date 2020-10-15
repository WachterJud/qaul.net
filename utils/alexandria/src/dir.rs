//! Directory helper to create and manage Alexandria instances

use crate::error::Result;
use std::{fs, path::PathBuf};

/// Metadata for where things are stored
pub(crate) struct Dirs {
    /// The root path, contains metadata
    root: PathBuf,
}

#[allow(unused)]
impl Dirs {
    pub(crate) fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self { root: root.into() }
    }

    pub(crate) fn scaffold(&self) -> Result<()> {
        fs::create_dir_all(&self.root)?;
        fs::create_dir(self.records())?;
        fs::create_dir(self.meta())?;
        fs::create_dir(self.cache())?;
        Ok(())
    }

    /// Return the records directory in the library
    pub(crate) fn records(&self) -> PathBuf {
        self.root.join("records")
    }

    /// Return the meta directory in the library
    pub(crate) fn meta(&self) -> PathBuf {
        self.root.join("meta")
    }

    /// Return the cache directory in the library
    pub(crate) fn cache(&self) -> PathBuf {
        self.root.join("cache")
    }
}

#[test]
fn scaffold_lib() -> Result<()> {
    use std::path::Path;
    use tempfile::tempdir;

    let root = tempdir().unwrap();
    let mut offset = root.path().to_path_buf();
    offset.push("library");

    let d = Dirs::new(offset.clone());
    d.scaffold()?;

    assert!(Path::new(dbg!(&offset.join("records"))).exists());
    assert!(Path::new(dbg!(&offset.join("meta"))).exists());
    assert!(Path::new(dbg!(&offset.join("cache"))).exists());
    Ok(())
}
