//! rs-auth CLI library entrypoint.

pub mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rs-auth", about = "rs-auth CLI")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Migrate {
        #[arg(long, env = "DATABASE_URL")]
        database_url: String,
    },
    Generate {
        name: String,
    },
    Cleanup {
        #[arg(long, env = "DATABASE_URL")]
        database_url: String,
    },
}

pub async fn run() -> anyhow::Result<()> {
    match Cli::parse().command {
        Commands::Migrate { database_url } => commands::migrate::run(&database_url).await,
        Commands::Generate { name } => commands::generate::run(&name),
        Commands::Cleanup { database_url } => commands::cleanup::run(&database_url).await,
    }
}
