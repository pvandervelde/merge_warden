//! # Merge Warden CLI
//!
//! Command-line interface for validating pull requests against configured rules.
//!
//! This binary provides a CLI interface to the Merge Warden functionality,
//! allowing users to validate pull requests, manage configuration, and
//! authenticate with Git providers from the command line.
//!
//! # Commands
//!
//! - `checkpr` - Validate a pull request against configured rules
//! - `config` - Manage configuration files and settings
//! - `auth` - Authenticate with Git providers (GitHub, GitLab, etc.)
//!
//! # Examples
//!
//! ```bash
//! # Check a pull request
//! merge-warden checkpr --repo owner/repo --pr-number 123
//!
//! # Initialize configuration
//! merge-warden config init
//!
//! # Authenticate with GitHub
//! merge-warden auth github --token <token>
//! ```

#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{error, info, instrument};

/// Command implementations for the CLI.
mod commands;

/// Configuration management for the CLI.
mod config;

/// Error types specific to the CLI.
mod errors;

use commands::{auth::AuthCommands, check_pr::CheckPrArgs, config_cmd::ConfigCommands};
use errors::CliError;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Command-line interface structure for Merge Warden.
///
/// This struct defines the top-level CLI interface using clap's derive API.
/// It includes global options like verbose logging and the main command structure.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// The subcommand to execute
    #[command(subcommand)]
    command: Commands,
}

/// Available commands for the Merge Warden CLI.
///
/// This enum defines all the subcommands that can be executed through
/// the CLI interface. Each variant corresponds to a different area of
/// functionality within Merge Warden.
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

/// Main entry point for the Merge Warden CLI.
///
/// This function initializes logging, parses command-line arguments,
/// and dispatches to the appropriate command handler based on the
/// user's input.
///
/// # Returns
///
/// Returns `Ok(())` on successful execution, or a `CliError` if any
/// operation fails.
///
/// # Errors
///
/// This function can return errors in the following cases:
/// - Command parsing failures
/// - Command execution failures
/// - Configuration errors
/// - Authentication errors
/// - Network or API errors during PR validation
///
/// # Examples
///
/// The function is called automatically when the binary is executed:
///
/// ```bash
/// merge-warden checkpr --repo owner/repo --pr-number 123
/// ```
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
