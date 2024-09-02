use anyhow::anyhow;

pub fn handler(name: &str, version: &Option<String>) -> anyhow::Result<()> {
    let server_path = axiom::server::get_server_path(name.to_owned())?;

    if !server_path.try_exists()? {
        return Err(anyhow!("Server {name} does not exist"));
    }

    let versions = axiom::version::versions()?;
    let releases = axiom::version::releases(&versions)?;
    // Get supplied version or use latest release by default
    let version = match version.to_owned() {
        Some(version) => version,
        None => releases
            .last()
            .ok_or_else(|| anyhow!("No releases found"))?
            .id
            .to_owned(),
    };

    if !releases.iter().any(|v| v.id == version) {
        return Err(anyhow!("'{version}' is not a valid version"));
    }

    log::info!("Attempting to update to Minecraft version {version}...");

    let builds = axiom::paper::get_builds(&version)?;
    let build = builds.last().ok_or_else(|| anyhow!("No builds found"))?;

    let jar_filename = axiom::paper::get_filename(&version, &build)?;
    let jar_filepath = axiom::fs::get_jars_path()?.join(&jar_filename);

    // If the server `.jar` does not exist, download and save it
    if !jar_filepath.try_exists()? {
        let server_jar = axiom::paper::get_server_jar(&jar_filename)?;
        std::fs::write(&jar_filepath, server_jar.data)?;
    } else {
        log::info!("Using cached server `.jar`: {}", jar_filepath.display());
    }

    let server_jar = server_path.join("server.jar");

    // These require **Create Symbolic Links Privilege** on Windows
    if server_jar.try_exists()? {
        symlink::remove_symlink_file(&server_jar)?;
    }

    if let Err(why) = symlink::symlink_file(&jar_filepath, &server_jar) {
        log::error!("Failed to link to new server `.jar`: {why}");
        log::error!("Attempting to unlink and re-link the file...");
        symlink::remove_symlink_file(&server_jar)?;
        symlink::symlink_file(&jar_filepath, &server_jar)?;
        log::info!("Successfully linked to new server `.jar`");
    }

    log::info!("{name} is now running Minecraft {version}!");
    Ok(())
}
