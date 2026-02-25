use clap::{Parser, Subcommand};
use tracing::{info};

use auria::{config::AppConfig, AuriaAgent};

#[derive(Parser, Debug)]
#[command(name="auria", version, about="AURIA Agent (controller) — production skeleton")]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Start HTTP API server
    Serve {
        /// Bind address, e.g. 127.0.0.1:8787
        #[arg(long)]
        bind: Option<String>,
    },
    /// Print effective configuration (after env overrides)
    Config,
    /// Perform a lightweight connectivity check against configured Auria Nodes
    Check,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    auria::telemetry::init_tracing();

    let cli = Cli::parse();
    let mut cfg = AppConfig::load()?;

    match cli.cmd {
        Command::Serve { bind } => {
            if let Some(b) = bind { cfg.bind = b; }
            let agent = AuriaAgent::new(cfg.clone()).await?;
            info!("starting auria agent on {}", cfg.bind);
            auria::api::serve(cfg, agent).await?;
        }
        Command::Config => {
            println!("{}", serde_json::to_string_pretty(&cfg)?);
        }
        Command::Check => {
            let agent = AuriaAgent::new(cfg.clone()).await?;
            agent.check_nodes().await?;
            info!("node connectivity OK");
        }
    }

    Ok(())
}
