use std::path::{Path, PathBuf};

use anyhow::Result;
use tokio::fs;

/// Crystal store - reads and writes markdown crystal files
pub struct CrystalStore {
    base_dir: PathBuf,
}

impl CrystalStore {
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    /// Read a crystal file by name (without .md extension)
    pub async fn read(&self, name: &str) -> Result<Option<String>> {
        let path = self.base_dir.join(format!("{}.md", name));
        if path.exists() {
            Ok(Some(fs::read_to_string(&path).await?))
        } else {
            Ok(None)
        }
    }

    /// Write/update a crystal file
    pub async fn write(&self, name: &str, content: &str) -> Result<()> {
        fs::create_dir_all(&self.base_dir).await?;
        let path = self.base_dir.join(format!("{}.md", name));
        fs::write(&path, content).await?;
        Ok(())
    }

    /// List all crystal files
    pub async fn list(&self) -> Result<Vec<String>> {
        let mut crystals = vec![];
        if !self.base_dir.exists() {
            return Ok(crystals);
        }

        let mut entries = fs::read_dir(&self.base_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".md") {
                crystals.push(name.trim_end_matches(".md").to_string());
            }
        }

        crystals.sort();
        Ok(crystals)
    }

    /// Append a section to a crystal file
    pub async fn append_section(&self, name: &str, section: &str) -> Result<()> {
        let current = self.read(name).await?.unwrap_or_default();
        let updated = format!("{}\n\n{}", current.trim(), section);
        self.write(name, &updated).await
    }
}
