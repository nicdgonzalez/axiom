use super::BASE_URL;
use super::Build;
use super::RequestError;

/// Represents a Minecraft version supported by PaperMC.
#[derive(Debug, Clone)]
pub struct Version(String);

impl Version {
    /// Represents a version of Minecraft supported by PaperMC.
    ///
    /// # Safety
    ///
    /// The caller is responsible for ensuring the value of `version` is a valid Minecraft version.
    /// An invalid `version` would likely result in errors when making calls to PaperMC.
    pub fn new(version: String) -> Self {
        Self(version)
    }

    /// Returns a reference to the underlying version string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get all of the available builds for the current version.
    ///
    /// This function sends a GET request to PaperMC to get a list of all available builds
    /// for this version of Minecraft.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - There is a problem sending the request to PaperMC.
    /// - Reading the response body times out.
    pub fn builds(&self) -> Result<Vec<Build>, RequestError> {
        let url = format!("{}/projects/paper/versions/{}/builds", BASE_URL, self.0);
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&url)
            .send()
            .and_then(|response| response.error_for_status())
            .map_err(RequestError::request_failed)?;

        debug_assert!(response.status().is_success());

        let text = response.text().map_err(RequestError::response_timed_out)?;

        #[derive(serde::Deserialize)]
        struct Response {
            builds: Vec<Build>,
        }

        let data: Response =
            serde_json::from_str(&text).map_err(RequestError::parse_response_failed)?;

        let builds = data
            .builds
            .into_iter()
            .map(|b| b.with_version(self.0.to_owned()))
            .collect();

        Ok(builds)
    }
}
