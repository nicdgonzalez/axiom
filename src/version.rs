//!

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionKind {
    OldAlpha,
    OldBeta,
    Snapshot,
    Release,
}

///
#[derive(Debug, Clone)]
pub struct Version {
    pub id: String,
    pub kind: VersionKind,
}

///
pub fn versions() -> anyhow::Result<Vec<Version>> {
    let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

    let client = reqwest::blocking::Client::new();
    let response = client.get(url).send()?.text()?;

    let data: serde_json::Value = serde_json::from_str(&response)?;
    let data = data.as_object().expect("expected JSON object");

    let versions: Vec<Version> = data
        .get("versions")
        .expect("expected field 'versions'")
        .as_array()
        .expect("expected 'versions' to be an array")
        .iter()
        .map(|v| {
            let version = v.as_object().expect("expected JSON object");

            let id = version
                .get("id")
                .expect("expected field 'id'")
                .as_str()
                .expect("expected 'id' to be a string")
                .to_owned();

            let kind = match version
                .get("type")
                .expect("expected field 'type'")
                .as_str()
                .expect("expected 'type' to be a string")
            {
                "old_alpha" => VersionKind::OldAlpha,
                "old_beta" => VersionKind::OldBeta,
                "snapshot" => VersionKind::Snapshot,
                "release" => VersionKind::Release,
                _ => unreachable!(),
            };

            Version { id, kind }
        })
        .collect();

    Ok(versions)
}

///
pub fn releases(versions: &[Version]) -> anyhow::Result<Vec<Version>> {
    Ok(versions
        .iter()
        .filter(|v| v.kind == VersionKind::Release)
        .rev()
        .cloned()
        .collect())
}
