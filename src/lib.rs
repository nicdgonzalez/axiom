// Axiom is a collection of tools for managing Minecraft servers.
// Copyright (C) 2024  Nicolas "nicdgonzalez" Gonzalez
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! # Axiom
//!
//! Axiom is a collection of tool for managing Minecraft servers.
//!
//! TODO

use anyhow::anyhow;

pub mod tmux;

pub fn init() -> anyhow::Result<()> {
    let backups = get_backups_path()?;
    let jars = get_jars_path()?;
    let servers = get_servers_path()?;

    for dir in [backups, jars, servers].iter() {
        std::fs::create_dir_all(dir)?;
    }

    Ok(())
}

pub fn normalize_server_name(name: &str) -> String {
    static MAX_LENGTH: u8 = 255; // Max filename length on Windows and Linux
    let normalized = name
        .trim()
        .chars()
        .take(MAX_LENGTH as usize)
        .map(|c| {
            // Invalid characters are replaced with a dash.
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .to_lowercase();

    normalized
}

pub fn get_axiom_path() -> anyhow::Result<std::path::PathBuf> {
    let axiom = dirs::data_dir()
        .ok_or_else(|| anyhow!("unable to get the data directory"))?
        .join("axiom");

    Ok(axiom)
}

pub fn get_backups_path() -> anyhow::Result<std::path::PathBuf> {
    let backups = get_axiom_path()?.join("backups");
    Ok(backups)
}

pub fn get_jars_path() -> anyhow::Result<std::path::PathBuf> {
    let jars = get_axiom_path()?.join("jars");
    Ok(jars)
}

pub fn get_servers_path() -> anyhow::Result<std::path::PathBuf> {
    let servers = get_axiom_path()?.join("servers");
    Ok(servers)
}

pub fn get_servers_dirs() -> anyhow::Result<Vec<std::fs::DirEntry>> {
    let servers: Vec<std::fs::DirEntry> = get_servers_path()?
        .read_dir()?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_ok_and(|ft| ft.is_dir()))
        .collect();

    Ok(servers)
}

pub fn get_server_path(name: &str) -> anyhow::Result<std::path::PathBuf> {
    let server = get_servers_path()?.join(name);
    Ok(server)
}

pub fn get_server_backups_path(name: &str) -> anyhow::Result<std::path::PathBuf> {
    let backups = get_backups_path()?.join(name);
    Ok(backups)
}

pub fn validate_server_exists(name: &str) -> anyhow::Result<(String, std::path::PathBuf)> {
    let name = normalize_server_name(name);
    let server = get_server_path(&name)?;

    if !server.try_exists()? {
        return Err(anyhow!("server with name '{name}' not found"));
    }

    Ok((name, server))
}

pub fn validate_server_not_exists(name: &str) -> anyhow::Result<(String, std::path::PathBuf)> {
    let name = normalize_server_name(name);
    let server = get_server_path(&name)?;

    if server.try_exists()? {
        return Err(anyhow!("server with name '{name}' already exists"));
    }

    Ok((name, server))
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum VersionKind {
//     OldAlpha,
//     OldBeta,
//     Snapshot,
//     Release,
// }
//
// #[derive(Debug, Clone)]
// pub struct Version {
//     pub id: String,
//     pub kind: VersionKind,
// }
//
// pub fn get_minecraft_versions() -> anyhow::Result<Vec<Version>> {
//     static URL: &'static str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
//
//     let client = reqwest::blocking::Client::new();
//     let response = client.get(URL).send()?.text()?;
//
//     let data: serde_json::Value = serde_json::from_str(&response)?;
//     let data = data.as_object().expect("expected JSON object");
//
//     let versions: Vec<Version> = data
//         .get("versions")
//         .expect("expected field 'versions'")
//         .as_array()
//         .expect("expected 'versions' to be an array")
//         .iter()
//         .map(|v| {
//             let version = v.as_object().expect("expected JSON object");
//
//             let id = version
//                 .get("id")
//                 .expect("expected field 'id'")
//                 .as_str()
//                 .expect("expected 'id' to be a string")
//                 .to_owned();
//
//             let kind = match version
//                 .get("type")
//                 .expect("expected field 'type'")
//                 .as_str()
//                 .expect("expected 'type' to be a string")
//             {
//                 "old_alpha" => VersionKind::OldAlpha,
//                 "old_beta" => VersionKind::OldBeta,
//                 "snapshot" => VersionKind::Snapshot,
//                 "release" => VersionKind::Release,
//                 _ => unreachable!(),
//             };
//
//             Version { id, kind }
//         })
//         .collect();
//
//     Ok(versions)
// }

static BASE_URL: &str = "https://api.papermc.io/v2";

pub fn get_paper_server_versions() -> anyhow::Result<Vec<String>> {
    let url = format!("{}/projects/paper", BASE_URL);

    let client = reqwest::blocking::Client::new();
    let response = client.get(url).send()?.text()?;

    let data: serde_json::Value = serde_json::from_str(&response)?;
    let data = data.as_object().expect("expected JSON object");

    let versions: Vec<String> = data
        .get("versions")
        .expect("expected field 'versions'")
        .as_array()
        .expect("expected 'versions' to be an array")
        .iter()
        .map(|v| {
            v.as_str()
                .expect("expected 'versions' to be an array of strings")
                .to_string()
        })
        .collect();

    Ok(versions)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelKind {
    Default,
    Experimental,
}

pub struct Build {
    pub version: String,
    pub build: i64,
    pub channel: ChannelKind,
    pub filename: String,
}

pub fn get_paper_build_latest(version: &str) -> anyhow::Result<Build> {
    let url = format!("{}/projects/paper/versions/{}/builds", BASE_URL, version);

    let client = reqwest::blocking::Client::new();
    let response = client.get(url).send()?.text()?;

    let data: serde_json::Value = serde_json::from_str(&response)?;
    let data = data.as_object().expect("expected JSON object");

    let builds = data
        .get("builds")
        .expect("expected field 'builds'")
        .as_array()
        .expect("expected 'builds' to be an array")
        .iter()
        .last()
        .map(|entry| {
            let build = entry
                .get("build")
                .expect("expected field 'build'")
                .as_i64()
                .expect("expected 'builds' to be an array of i64");
            let channel = match entry
                .get("channel")
                .expect("expected field 'channel'")
                .as_str()
                .expect("expected field 'channel' to be a string")
            {
                "default" => ChannelKind::Default,
                "experimental" => ChannelKind::Experimental,
                _ => unreachable!(),
            };
            let filename = entry
                .get("downloads")
                .expect("expected field 'downloads'")
                .as_object()
                .expect("expected 'downloads' to be a JSON object")
                .get("application")
                .expect("expected field 'application'")
                .as_object()
                .expect("expected 'application' to be a JSON object")
                .get("name")
                .expect("expected field 'name'")
                .as_str()
                .expect("expected 'name' to be a string")
                .to_string();

            Build {
                version: version.to_string(),
                build,
                channel,
                filename,
            }
        })
        .expect("no builds available");

    Ok(builds)
}

pub struct ServerJar {
    pub filename: String,
    pub data: Vec<u8>,
}

pub fn get_paper_server_jar(build: &Build) -> anyhow::Result<ServerJar> {
    let parts: Vec<&str> = build.filename.split('-').collect();
    let version = parts
        .get(1)
        .expect("expected filename format: paper-{version}-{build}.jar");
    let build_number = parts
        .get(2)
        .expect("expected filename format: paper-{version}-{build}.jar")
        .split_once('.')
        .expect("expected filename to end with `.jar`")
        .0;

    let url = format!(
        "{}/projects/paper/versions/{}/builds/{}/downloads/{}",
        BASE_URL, &version, &build_number, &build.filename
    );

    let client = reqwest::blocking::Client::new();
    let data = client
        .get(url)
        .timeout(std::time::Duration::from_secs(60 * 2))
        .send()?
        .bytes()?
        .to_vec();

    Ok(ServerJar {
        filename: build.filename.to_owned(),
        data,
    })
}
