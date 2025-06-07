use super::BASE_URL;
use crate::RequestError;

/// Represents an official release for a PaperMC Minecraft server JAR file.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Build {
    /// The version of Minecraft this build is intended for.
    #[serde(skip)]
    version: String,

    /// An incremental counter unique to each build that helps track the progress of releases.
    #[serde(rename = "build")]
    number: i32,

    /// Indicates the status of the build.
    channel: Channel,

    /// Contains information about the downloadable server JAR file associated with this build.
    downloads: Downloads,
}

/// Describes which channel a build was released under.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum Channel {
    /// Indicates a stable build.
    Default,
    /// Indicates an experimental build.
    Experimental,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct Downloads {
    application: Application,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct Application {
    name: String,
}

impl Build {
    pub(crate) fn with_version(self, version: String) -> Self {
        Self { version, ..self }
    }

    /// Indicates the version of Minecraft this build is intended for.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Indicates the number of times a new release has been made for this version.
    pub fn number(&self) -> i32 {
        self.number
    }

    /// Indicates if the build was released under the default channel.
    pub fn stable(&self) -> bool {
        self.channel == Channel::Default
    }

    /// Indicates if the build was released under the experimental channel.
    pub fn experimental(&self) -> bool {
        self.channel == Channel::Experimental
    }

    /// The original name of the server JAR file.
    pub fn download_name(&self) -> &str {
        &self.downloads.application.name
    }

    /// Gets the server JAR file and returns its contents as raw bytes.
    ///
    /// This function calls the PaperMC API to get the contents of server JAR file.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - ...
    ///
    /// # Examples
    ///
    /// ```no_run
    /// fn main() {
    ///     // 1. Get the bytes from PaperMC.
    ///     // 2. Write them to a file.
    ///     // 3. Run the file to generate the Minecraft server.
    /// }
    /// ```
    pub fn download(
        &self,
        timeout: std::time::Duration,
    ) -> Result<Vec<u8>, crate::error::RequestError> {
        assert!(
            !self.version.is_empty(),
            "use `with_version` to set the Minecraft version"
        );
        let url = format!(
            "{}/projects/paper/versions/{}/builds/{}/downloads/{}",
            BASE_URL, self.version, self.number, self.downloads.application.name
        );
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&url)
            .timeout(timeout)
            .send()
            .and_then(|response| response.error_for_status())
            .map_err(RequestError::request_failed)?;

        debug_assert!(response.status().is_success());

        let bytes = response
            .bytes()
            .map_err(RequestError::response_timed_out)?
            .to_vec();

        Ok(bytes)
    }
}
