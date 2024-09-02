//!

/// Takes the given name and ensures it can be used as a directory name.
fn sanitize_server_name(name: String) -> String {
    static MAX_LENGTH: u8 = 255; // Max filename length on Windows and Linux (I think)
    let sanitized = name
        .trim()
        .chars()
        .take(MAX_LENGTH as usize)
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .to_lowercase();

    sanitized
}

/// Get a vector of `std::fs::DirEntry` objects from the `servers` directory.
///
/// This function searches through the `servers` directory and filters out any
/// entries that are not directories.
///
/// # Errors
///
/// Returns an error if the `servers` directory can not be accessed.
///
/// # Examples
///
/// ```
/// fn main() -> anyhow::Result<()> {
///     let servers = axiom::server::get_server_dirs()?;
///
///     for (i, server) in servers.iter().enumerate() {
///         let name = &server.file_name();
///         let name = name.to_str().unwrap();
///         println!("{}. {}", i + 1, &name);
///     }
///     Ok(())
/// }
/// ```
pub fn get_server_dirs() -> anyhow::Result<Vec<std::fs::DirEntry>> {
    let servers: Vec<std::fs::DirEntry> = crate::fs::get_root_path()?
        .join("servers")
        .read_dir()?
        .filter_map(|e| e.ok())
        .filter(|f| f.file_type().is_ok_and(|ft| ft.is_dir()))
        .collect();

    Ok(servers)
}

/// Get the path to a server in the `servers` directory.
///
/// This function returns a path to a directory in the `servers` directory.
/// The server name is sanitized prior to being added to the servers path.
///
/// # Errors
///
/// Returns an error if the `servers` directory can not be accessed.
///
/// # Examples
///
/// ```
/// fn main() -> anyhow::Result<()> {
///     let name = String::from("My World");
///     assert!(axiom::server::get_server_path(name).is_ok());
///     Ok(())
/// }
/// ```
///
/// You are responsible for validating whether the server directory actually
/// exists, based on your own needs:
///
/// ```no_run
/// fn main() -> anyhow::Result<()> {
///     let name = String::from("My World");
///     let server_path = axiom::server::get_server_path(name.clone())?;
///     
///     if !server_path.try_exists()? {
///         eprintln!("Server {name} does not exist...");
///     } else {
///         println!("Server {name} exists!");
///     }
///
///     Ok(())
/// }
/// ```
pub fn get_server_path(name: String) -> anyhow::Result<std::path::PathBuf> {
    let servers_path = crate::fs::get_root_path()?.join("servers");
    let name = sanitize_server_name(name);
    let server_path = servers_path.join(&name);
    Ok(server_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_server_name() {
        let name = String::from("     My World");
        assert_eq!(&sanitize_server_name(name), "my-world");

        let name = String::from("MyWorld");
        assert_eq!(&sanitize_server_name(name), "myworld");

        let name = String::from("My World");
        assert_eq!(&sanitize_server_name(name), "my-world");

        let name = String::from("僕の世界");
        assert_eq!(&sanitize_server_name(name), "僕の世界");

        let name = String::from("\\/*^%$");
        assert_eq!(&sanitize_server_name(name), "------");

        let name = String::from(" ?!My World!?");
        assert_eq!(&sanitize_server_name(name), "--my-world--");
    }
}
