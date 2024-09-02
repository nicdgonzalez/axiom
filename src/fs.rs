//!

// TODO: Split this file into separate files that implement the functionality
// for the specified directory.

use once_cell::sync::Lazy;

/// Path to the `.axiom` directory in the user's home directory
///
/// # Errors
///
/// # Examples
pub fn get_root_path() -> anyhow::Result<&'static std::path::Path> {
    static ROOT_PATH: Lazy<std::path::PathBuf> = Lazy::new(|| {
        let home = home::home_dir().expect("Unable to determine user's home directory");
        home.join(".axiom")
    });
    Ok(ROOT_PATH.as_ref())
}

/// Path to the `backups` directory.
///
/// This is where a server's compressed backups are stored.
///
/// # Errors
///
/// # Examples
pub fn get_backups_path() -> anyhow::Result<std::path::PathBuf> {
    Ok(get_root_path()?.join("backups"))
}

/// Path to the `jars` directory.
///
/// This is where server `.jar` files are cached.
///
/// # Errors
///
/// # Examples
pub fn get_jars_path() -> anyhow::Result<std::path::PathBuf> {
    Ok(get_root_path()?.join("jars"))
}

/// Path to the `pipes` directory.
///
/// This is for communicating between Axiom and the Minecraft server.
///
/// # Errors
///
/// # Examples
pub fn get_pipes_path() -> anyhow::Result<std::path::PathBuf> {
    // TODO: Ensure only the "axiom" group has read/write access
    Ok(get_root_path()?.join("pipes"))
}

/// Path to the `servers` directory.
///
/// This is where all of the user's servers are stored.
///
/// # Errors
///
/// # Examples
pub fn get_servers_path() -> anyhow::Result<std::path::PathBuf> {
    Ok(get_root_path()?.join("servers"))
}
