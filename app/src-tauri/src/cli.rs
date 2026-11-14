//! CLI command handling
//!
//! Defines command-line interface and handles CLI subcommands.
//!
//! This module is responsible for:
//! - CLI argument parsing (clap)
//! - User interaction (y/n confirmation, progress display)
//! - Delegating to business logic modules (updater)

use clap::{Parser, Subcommand};
use std::io::{self, Write};

use crate::updater;

/// uti - Double Ctrl hotkey desktop tool
#[derive(Parser)]
#[command(name = "uti")]
#[command(about = "Desktop utility for toggling window visibility with double Ctrl press")]
#[command(version, long_version = env!("CARGO_PKG_VERSION"), disable_version_flag = true)]
pub struct Cli {
    /// Print version
    #[arg(short = 'v', short_alias = 'V', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    /// Start minimized (used by autostart)
    #[arg(long)]
    pub minimized: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Check for updates and install if available
    Update {
        /// Check only, don't install
        #[arg(long)]
        check: bool,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

/// Prompt user for y/n confirmation
///
/// Returns true if user confirms, false otherwise.
fn confirm(prompt: &str) -> bool {
    print!("{} [y/N]: ", prompt);
    io::stdout().flush().ok();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }

    input.trim().eq_ignore_ascii_case("y")
}

/// Handle CLI update command
async fn handle_update_command(check_only: bool, skip_confirm: bool) {
    let current_version = env!("CARGO_PKG_VERSION");
    println!("Current version: {}", current_version);
    println!("Checking for updates...");

    match updater::check_for_updates(current_version).await {
        Ok(result) => {
            if result.update_available {
                println!(
                    "Update available: {} -> {}",
                    result.current_version, result.latest_version
                );

                if check_only {
                    println!("Run 'uti update' to install the update.");
                    return;
                }

                if result.uti_rpm_url.is_none() && result.daemon_rpm_url.is_none() {
                    println!("No RPM packages found in the release.");
                    return;
                }

                // Confirm before installing (unless -y flag is passed)
                if !skip_confirm && !confirm("Do you want to install the update?") {
                    println!("Update cancelled.");
                    return;
                }

                println!();
                println!("Downloading and installing update...");
                println!("(You may be prompted for your password)");
                println!();

                match updater::perform_update(&result).await {
                    Ok(()) => {
                        println!("Update installed successfully!");
                        println!();
                        // Red bold warning box
                        println!("\x1b[1;31m+--------------------------------------------------------------+\x1b[0m");
                        println!("\x1b[1;31m|  WARNING: YOU MUST LOG OUT AND LOG BACK IN TO APPLY CHANGES  |\x1b[0m");
                        println!("\x1b[1;31m+--------------------------------------------------------------+\x1b[0m");
                        println!();
                    }
                    Err(e) => {
                        eprintln!("Update failed: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                println!("You are running the latest version.");
            }
        }
        Err(e) => {
            eprintln!("Failed to check for updates: {}", e);
            std::process::exit(1);
        }
    }
}

/// Run CLI command if subcommand is specified
///
/// Returns `true` if a subcommand was handled (caller should exit),
/// `false` if no subcommand (caller should run GUI).
pub fn run_command(cli: &Cli) -> bool {
    if let Some(ref command) = cli.command {
        match command {
            Commands::Update { check, yes } => {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
                rt.block_on(handle_update_command(*check, *yes));
                return true;
            }
        }
    }
    false
}
