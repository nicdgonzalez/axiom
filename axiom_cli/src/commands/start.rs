pub fn handler(name: &str) -> anyhow::Result<()> {
    let server_path = axiom::server::get_server_path(name.to_owned())?;

    if !server_path.try_exists()? {
        return Err(anyhow::anyhow!("Server {name} does not exist"));
    }

    log::info!("Attempting to start server...");

    let child = std::process::Command::new("java")
        .current_dir(&server_path)
        .args([
            "-Xms2G",
            "-Xmx2G",
            "-XX:+UseG1GC",
            "-XX:+ParallelRefProcEnabled",
            "-XX:MaxGCPauseMillis=200",
            "-XX:+UnlockExperimentalVMOptions",
            "-XX:+DisableExplicitGC",
            "-XX:+AlwaysPreTouch",
            "-XX:G1NewSizePercent=30",
            "-XX:G1MaxNewSizePercent=40",
            "-XX:G1HeapRegionSize=8M",
            "-XX:G1ReservePercent=20",
            "-XX:G1HeapWastePercent=5",
            "-XX:G1MixedGCCountTarget=4",
            "-XX:InitiatingHeapOccupancyPercent=15",
            "-XX:G1MixedGCLiveThresholdPercent=90",
            "-XX:G1RSetUpdatingPauseTimePercent=5",
            "-XX:SurvivorRatio=32",
            "-XX:+PerfDisableSharedMem",
            "-XX:MaxTenuringThreshold=1",
            "-Dusing.aikars.flags=https://mcflags.emc.gs",
            "-Daikars.new.flags=true",
            "-jar",
            "server.jar",
            "--nogui",
        ])
        .stdout(std::process::Stdio::null())
        .stdin(std::process::Stdio::null())
        .spawn()?;

    log::warn!("Detecting the Axiom plugin is not yet implemented; to stop the server manually, use this PID: {}", child.id());

    Ok(())
}
