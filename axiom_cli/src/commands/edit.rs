//! Implementation of the `edit` command.

use anyhow::anyhow;

pub fn handler(name: &str) -> anyhow::Result<()> {
    let server_path = axiom::server::get_server_path(name.to_owned())?;
    let server_properties = server_path.join("server.properties");

    if !server_properties.try_exists()? {
        log::error!(
            "Server {name} was likely not initialized properly; missing server.properties file"
        );
        return Err(anyhow!("Missing file: {}", server_properties.display()));
    }

    let editor = std::env::var("EDITOR")?;
    let mut command = std::process::Command::new(&editor);
    let mut child = match command
        .arg(server_properties.as_path())
        .current_dir(&server_path)
        .spawn()
    {
        Ok(v) => v,
        Err(why) => {
            return Err(anyhow!(
                "Failed to open `server.properties` in the default editor: {why}"
            ))
        }
    };

    let exit_status = child.wait()?;

    if !exit_status.success() {
        match exit_status.code() {
            Some(code) => log::error!("Editor command exited with a non-zero exit code: {}", code),
            None => log::error!("Editor command exited abnormally"),
        }

        log::warn!(
            "Unable to determine if the file's changes were saved: {}",
            server_properties.display()
        );
    }

    log::info!("Successfully updated server.properties for {name}");
    Ok(())
}
