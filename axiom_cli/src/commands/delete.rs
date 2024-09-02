use std::io::Write;

pub fn handler(name: &str, assume_yes: &bool) -> anyhow::Result<()> {
    let server = axiom::server::get_server_path(name.to_owned())?;

    if !assume_yes {
        print!("Are you sure you want to delete {name}? (y/N): ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "y" {
            return Ok(());
        }
    }

    std::fs::remove_dir_all(&server)?;
    log::info!("{name} has been deleted");
    Ok(())
}
