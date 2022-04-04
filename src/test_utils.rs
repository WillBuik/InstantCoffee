use std::{path::{PathBuf, Path}, fs::{metadata, remove_file}};

pub struct CleanupFile (PathBuf);

impl From<&Path> for CleanupFile {
    fn from(path: &Path) -> Self {
        if let Ok(_) = metadata(&path) {
            panic!("CleanupFile '{}' already exists", path.to_string_lossy());
        }
        return CleanupFile(PathBuf::from(path));
    }
}

impl Drop for CleanupFile {
    fn drop(&mut self) {
        let _ = remove_file(&self.0);
    }
}
