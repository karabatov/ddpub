mod config;
mod dd;
mod l10n;
mod layout;
mod notes;

use clap::{Parser, Subcommand};
use std::time::Instant;

const DEFAULT_PORT: u16 = 44234;

#[derive(Parser)]
#[command(about = "ddpub is a tool to serve one set of notes as many websites.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Check config
    Check {
        /// Directory that has `config.toml`
        #[arg(long, default_value = ".")]
        config: String,
        /// Directory that stores notes
        #[arg(long, default_value = ".")]
        notes: String,
    },
    /// Serve notes
    Serve {
        /// Directory that has `config.toml`
        #[arg(long, default_value = ".")]
        config: String,
        /// Directory that stores notes
        #[arg(long, default_value = ".")]
        notes: String,
        /// Port to serve notes
        #[arg(long, default_value_t = DEFAULT_PORT)]
        port: u16,
    },
}

#[tokio::main]
async fn main() {
    let start_time = Instant::now();
    let cli = Cli::parse();

    let (config_dir, notes_dir, port, serve) = match &cli.command {
        Command::Check { config, notes } => (config.clone(), notes.clone(), 0, false),
        Command::Serve { config, notes, port } => (config.clone(), notes.clone(), *port, true),
    };

    let cfg = match crate::config::Website::new(&config_dir) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Couldn't load website config: {e}");
            std::process::exit(1);
        }
    };

    let store = match notes::multistore::MultiStore::new(&cfg, &notes_dir) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Couldn't load notes: {e}");
            std::process::exit(1);
        }
    };

    let router = match notes::multirouter::MultiRouter::new(&cfg, &store) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Could not create router: {e}");
            std::process::exit(1);
        }
    };

    eprintln!(
        "Config OK. Startup took {:?}.",
        start_time.elapsed()
    );

    if !serve {
        std::process::exit(0);
    }

    eprintln!("Starting server...");
    eprintln!("In your browser, open: http://localhost:{port}");

    let app = router.into_axum_router();
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
