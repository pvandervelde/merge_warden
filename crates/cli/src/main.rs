use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{error, info, instrument};

mod commands;
mod config;
mod errors;

use commands::{auth::AuthCommands, check_pr::CheckPrArgs, config_cmd::ConfigCommands};
use errors::CliError;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Merge Warden CLI - Validate pull requests against configured rules
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a pull request against configured rules
    #[command(name = "checkpr")]
    CheckPr(CheckPrArgs),

    /// Manage configuration
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Authenticate with Git providers
    #[command(subcommand)]
    Auth(AuthCommands),
}

#[tokio::main]
#[instrument]
async fn main() -> Result<(), CliError> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(fmt::layer().pretty())
        .with(EnvFilter::from_env("MERGE_WARDEN_LOG"))
        .init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Set verbose logging if requested
    if cli.verbose {
        info!("Verbose mode enabled");
    }

    // Execute the appropriate command
    match cli.command {
        Commands::CheckPr(args) => match commands::check_pr::execute(args).await {
            Ok(result) => {
                return Ok(result);
            }
            Err(e) => {
                error!("Error validating pull requests: {}", e);
                return Err(e);
            }
        },
        Commands::Config(cmd) => {
            if let Err(e) = commands::config_cmd::execute(cmd).await {
                error!("Error executing config command: {}", e);
                return Err(e);
            }
        }
        Commands::Auth(cmd) => {
            if let Err(e) = commands::auth::execute(cmd).await {
                error!("Error executing auth command: {}", e);
                return Err(e);
            }
        }
    }

    Ok(())
}
