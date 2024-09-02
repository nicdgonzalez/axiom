use std::io::Write;

pub fn handler(name: &str, version: &Option<String>, accept_eula: &bool) -> anyhow::Result<()> {
    let server_path = axiom::server::get_server_path(name.to_owned())?;

    if server_path.try_exists()? {
        return Err(anyhow::anyhow!("Server {name} already exists"));
    } else {
        log::info!("Creating new directory: {}", server_path.display());
        std::fs::create_dir_all(&server_path)?;
    }

    // Get the server `.jar` for the requested version
    match super::update::handler(&name, &version) {
        Ok(_) => {}
        Err(why) => {
            log::error!("Failed to update server.jar; removing half-initialized directory...");
            super::delete::handler(&name, &true)?;
            return Err(why);
        }
    }

    log::info!("Running the server to generate intial files...");
    let mut command = std::process::Command::new("java");
    let mut child = match command
        .args(["-jar", server_path.join("server.jar").to_str().unwrap()])
        .current_dir(&server_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(value) => value,
        Err(why) => {
            log::error!("Failed to run server.jar; removing half-initialized directory...");
            super::delete::handler(&name, &true)?;
            return Err(anyhow::anyhow!(
                "Failed to execute command `java` in child process: {why}"
            ));
        }
    };

    // Running the server for the first time always fails,
    // telling the user to accept the Minecraft EULA before continuing
    child.wait().expect("Command wasn't running");

    if !accept_eula {
        println!(
            "You must accept the Minecraft EULA before continuing: https://aka.ms/MinecraftEULA"
        );
        print!("Accept and continue? (y/N): ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "y" {
            log::error!(
                "User did not accept the Minecraft EULA; removing half-initialized directory..."
            );
            super::delete::handler(&name, &true)?;
            return Err(anyhow::anyhow!("User did not agree to the Minecraft EULA"));
        }
    }

    let eula_txt = server_path.join("eula.txt");
    let text = std::fs::read_to_string(&eula_txt)?;
    std::fs::write(&eula_txt, text.replace("eula=false", "eula=true"))?;

    // TODO: Auto-install the Axiom plugin for communicating with the server

    log::info!("{name} is ready!");
    Ok(())
}
