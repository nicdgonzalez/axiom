//! # Axiom

pub mod fs;
pub mod paper;
pub mod server;
pub mod version;
pub mod backups;

use anyhow::anyhow;

/// Get the path to the `.axiom` directory in the user's home directory.
///
/// # Errors
///
/// Returns an error if the user's home directory can not be determined.
/// This is usually `$HOME` on Linux, `%userprofile%` on Windows.
pub fn get_root_path() -> anyhow::Result<std::path::PathBuf> {
    let root_path = home::home_dir()
        .ok_or_else(|| anyhow!("Unable to determine the user's home directory"))?
        .join(".axiom");
    Ok(root_path)
}

///
pub fn init() -> anyhow::Result<()> {
    let subdirs = ["backups", "jars", "pipes", "servers"];
    let root = get_root_path()?;

    for subdir in subdirs.iter() {
        let path = root.join(&subdir);
        std::fs::create_dir_all(&path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_root_path() {
        assert!(get_root_path().is_ok());
    }
}