use anyhow::Context;
use trust_dns_resolver::Resolver;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};

use crate::commands::status::Status;

#[derive(Debug, Clone, clap::Args)]
pub struct StatusExt {
    /// The IP address or hostname of the target Minecraft server.
    #[arg(long, short = 'H')]
    pub(crate) hostname: String,

    /// The port number on which the Minecraft server is listening for connections.
    #[arg(long, short = 'p')]
    pub(crate) port: Option<u16>,

    /// The maximum number of seconds to wait before failing to connect to the server.
    #[arg(long, default_value = "10")]
    pub(crate) timeout: u64,
}

impl crate::commands::Run for StatusExt {
    fn run(&self, ctx: &mut crate::context::Context) -> Result<(), crate::error::Error> {
        let domain = format!("_minecraft._tcp.{}", self.hostname);
        let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();

        let (hostname, port) = resolver
            .srv_lookup(&domain)
            .map(|records| {
                records
                    .into_iter()
                    .next()
                    .map(|record| (record.target().to_string(), record.port()))
                    .expect("expected at least one result from srv resolver")
            })
            .with_context(|| "failed to resolve hostname")?;

        let temporary_directory = tempdir::TempDir::new("axiom")
            .with_context(|| "failed to create temporary directory")?;
        let file_path = temporary_directory.path().join("Axiom.toml");
        let contents = format!(
            r#"[server]
version = "1.21.5"

[properties]
server-ip = "{hostname}"
server-port = {port}
"#
        );
        std::fs::write(&file_path, &contents)
            .with_context(|| "failed to write to temporary Axiom.toml")?;

        Status {
            timeout: self.timeout,
        }
        .run(ctx)
    }
}
