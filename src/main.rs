mod config;
mod crypto;
mod db;
mod daemon;
mod tui;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use config::Config;
use daemon::Daemon;

#[derive(Parser)]
#[command(name = "secapp", version, about = "Secure environment keystore for Linux")]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short = 'v', global = true, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the secapp daemon with TUI interface
    Start,

    /// Start the daemon in background (no TUI)
    Daemon,

    /// Encrypt a specific file
    Encrypt {
        /// Path to the file to encrypt
        path: PathBuf,
    },

    /// Decrypt a specific file
    Decrypt {
        /// Path to the file to decrypt
        path: PathBuf,
    },

    /// Add a directory to the protected list
    AddPath {
        /// Directory path to protect
        path: PathBuf,
    },

    /// Remove a directory from the protected list
    RemovePath {
        /// Directory path to remove
        path: PathBuf,
    },

    /// List protected directories and their status
    List,

    /// Show access logs
    Logs {
        /// Maximum number of log entries to show
        #[arg(short, long, default_value = "50")]
        limit: i64,
    },

    /// Manage access policies
    #[command(subcommand)]
    Policy(PolicyCommands),

    /// Lock the keystore (clear master key from memory)
    Lock,

    /// Initialize secapp configuration
    Init,
}

#[derive(Subcommand)]
enum PolicyCommands {
    /// Add a new access policy
    Add {
        /// Path glob pattern (e.g., /home/user/.ssh/*)
        path: String,
        /// Process name pattern (e.g., ssh*, or * for all)
        process: String,
        /// Action: allow, deny, or log
        #[arg(value_enum)]
        action: PolicyAction,
        /// Priority (higher = overrides)
        #[arg(short, long, default_value = "0")]
        priority: i64,
    },

    /// Remove a policy by ID
    Remove {
        /// Policy ID to remove
        id: i64,
    },

    /// List all policies
    List,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum PolicyAction {
    Allow,
    Deny,
    Log,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging based on verbosity
    let log_level = match cli.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level))
        )
        .init();

    match cli.command.unwrap_or(Commands::Start) {
        Commands::Start => run_start().await,
        Commands::Daemon => run_daemon().await,
        Commands::Encrypt { path } => run_encrypt(path).await,
        Commands::Decrypt { path } => run_decrypt(path).await,
        Commands::AddPath { path } => run_add_path(path).await,
        Commands::RemovePath { path } => run_remove_path(path).await,
        Commands::List => run_list().await,
        Commands::Logs { limit } => run_logs(limit).await,
        Commands::Policy(cmd) => run_policy(cmd).await,
        Commands::Lock => run_lock().await,
        Commands::Init => run_init().await,
    }
}

async fn run_start() -> Result<()> {
    let config = Config::load()?;
    let daemon = Daemon::new(config.clone()).await?;

    // Try to restore master key from kernel keyring
    daemon.try_keyring_restore().await?;

    tui::run_tui(config, daemon).await
}

async fn run_daemon() -> Result<()> {
    let config = Config::load()?;
    let daemon = Daemon::new(config.clone()).await?;

    // Try to restore from keyring
    daemon.try_keyring_restore().await?;

    if !daemon.is_unlocked().await {
        anyhow::bail!("Keystore is locked. Please unlock via 'secapp start' or provide a password.");
    }

    // Write PID file
    let pid_file = daemon::PidFile::new(config.daemon.pid_file.clone());
    pid_file.write()?;

    let result = daemon.run().await;

    // Clean up PID file
    let _ = pid_file.remove();

    result
}

async fn run_encrypt(_path: PathBuf) -> Result<()> {
    let config = Config::load()?;
    let daemon = Daemon::new(config.clone()).await?;

    if !daemon.is_unlocked().await {
        anyhow::bail!("Keystore is locked. Unlock first with 'secapp start'.");
    }

    // Direct file encryption via the interceptor requires the master key
    // This is a simplified interface for one-off encryption
    anyhow::bail!("Direct file encryption requires the daemon running. Use 'secapp start' to run the daemon.");
}

async fn run_decrypt(_path: PathBuf) -> Result<()> {
    let config = Config::load()?;
    let _daemon = Daemon::new(config.clone()).await?;

    anyhow::bail!("Direct file decryption requires the daemon running. Use 'secapp start' to run the daemon.");
}

async fn run_add_path(path: PathBuf) -> Result<()> {
    let mut config = Config::load()?;
    let abs_path = std::fs::canonicalize(&path)
        .with_context(|| format!("resolving path {}", path.display()))?;

    if config.add_protected_path(&abs_path) {
        config.save()?;
        println!("Added protected path: {}", abs_path.display());
    } else {
        println!("Path already protected: {}", abs_path.display());
    }
    Ok(())
}

async fn run_remove_path(path: PathBuf) -> Result<()> {
    let mut config = Config::load()?;
    let abs_path = std::fs::canonicalize(&path)
        .with_context(|| format!("resolving path {}", path.display()))?;

    if config.remove_protected_path(&abs_path) {
        config.save()?;
        println!("Removed protected path: {}", abs_path.display());
    } else {
        println!("Path not in protected list: {}", abs_path.display());
    }
    Ok(())
}

async fn run_list() -> Result<()> {
    let config = Config::load()?;
    let daemon = Daemon::new(config.clone()).await?;

    println!("Protected directories:");
    for path in &config.protected_paths {
        let exists = if path.exists() { "✓" } else { "✗" };
        println!("  {} {}", exists, path.display());
    }

    println!();
    println!("Tracked files:");
    let files = daemon.db().list_protected_files().await?;
    if files.is_empty() {
        println!("  (none)");
    } else {
        for file in &files {
            let status = if file.is_encrypted { "🔒" } else { "🔓" };
            println!(
                "  {} {} ({} bytes)",
                status, file.path, file.original_size
            );
        }
    }

    Ok(())
}

async fn run_logs(limit: i64) -> Result<()> {
    let config = Config::load()?;
    let daemon = Daemon::new(config.clone()).await?;

    let logs = daemon.db().list_access_logs(limit).await?;
    if logs.is_empty() {
        println!("No access logs found.");
        return Ok(());
    }

    println!("{:<5} {:<20} {:<10} {:<15} {:<8} {:<8} {}",
        "ID", "Time", "Action", "Process", "PID", "Allowed", "Path");
    println!("{}", "-".repeat(80));
    for log in &logs {
        let allowed = if log.allowed { "yes" } else { "no" };
        println!(
            "{:<5} {:<20} {:<10} {:<15} {:<8} {:<8} {}",
            log.id, log.timestamp, log.action, log.process_name, log.process_pid, allowed, log.file_path
        );
    }
    Ok(())
}

async fn run_policy(cmd: PolicyCommands) -> Result<()> {
    let config = Config::load()?;
    let daemon = Daemon::new(config.clone()).await?;
    let db = daemon.db();

    match cmd {
        PolicyCommands::Add { path, process, action, priority } => {
            let action_str = match action {
                PolicyAction::Allow => "allow",
                PolicyAction::Deny => "deny",
                PolicyAction::Log => "log",
            };
            let id = db.add_policy(&path, &process, action_str, priority).await?;
            println!("Added policy #{}: {} {} from {} (priority={})", id, action_str, path, process, priority);
        }
        PolicyCommands::Remove { id } => {
            if db.remove_policy(id).await? {
                println!("Removed policy #{}", id);
            } else {
                println!("Policy #{} not found", id);
            }
        }
        PolicyCommands::List => {
            let policies = db.list_policies().await?;
            if policies.is_empty() {
                println!("No policies configured.");
                return Ok(());
            }
            println!("{:<5} {:<10} {:<30} {:<20} {:<8} {}",
                "ID", "Action", "Path", "Process", "Priority", "Created");
            println!("{}", "-".repeat(80));
            for policy in &policies {
                println!(
                    "{:<5} {:<10} {:<30} {:<20} {:<8} {}",
                    policy.id, policy.action, policy.path_pattern,
                    policy.process_pattern, policy.priority, policy.created_at
                );
            }
        }
    }

    Ok(())
}

async fn run_lock() -> Result<()> {
    let config = Config::load()?;
    let daemon = Daemon::new(config.clone()).await?;
    daemon.lock().await?;
    println!("Keystore locked.");
    Ok(())
}

async fn run_init() -> Result<()> {
    let config = Config::default();
    let config_path = Config::config_dir()?.join("config.json");
    if config_path.exists() {
        println!("Configuration already exists at {}", config_path.display());
        println!("Delete it manually if you want to reinitialize.");
        return Ok(());
    }
    config.save()?;
    println!("Configuration initialized at {}", config_path.display());

    // Also initialize the database
    let db_path = Config::db_path()?;
    let _db = db::Database::open(&db_path).await?;
    println!("Database initialized at {}", db_path.display());
    Ok(())
}