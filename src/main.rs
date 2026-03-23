use clap::{Parser, Subcommand};
use ddpub::{MultiRouter, MultiStore, Website};
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
    /// Export a full static website to a directory
    Export {
        /// Directory that has `config.toml`
        #[arg(long, default_value = ".")]
        config: String,
        /// Directory that stores notes
        #[arg(long, default_value = ".")]
        notes: String,
        /// Clear the export directory before exporting
        #[arg(long)]
        force: bool,
        /// Directory to export the website to
        export_dir: String,
    },
}

#[tokio::main]
async fn main() {
    let start_time = Instant::now();
    let cli = Cli::parse();

    let (config_dir, notes_dir, port, serve, export) = match &cli.command {
        Command::Check { config, notes } => (config.clone(), notes.clone(), 0, false, None),
        Command::Serve { config, notes, port } => {
            (config.clone(), notes.clone(), *port, true, None)
        }
        Command::Export {
            config,
            notes,
            force,
            export_dir,
        } => (
            config.clone(),
            notes.clone(),
            0,
            false,
            Some((export_dir.clone(), *force)),
        ),
    };

    let cfg = match Website::new(&config_dir) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Couldn't load website config: {e}");
            std::process::exit(1);
        }
    };

    let store = match MultiStore::new(&cfg, &notes_dir) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Couldn't load notes: {e}");
            std::process::exit(1);
        }
    };

    let router = match MultiRouter::new(&cfg, &store) {
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

    if let Some((dir, force)) = export {
        let config_path = match std::fs::canonicalize(&config_dir) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Could not resolve config directory '{config_dir}': {e}");
                std::process::exit(1);
            }
        };
        let notes_path = std::fs::canonicalize(&notes_dir)
            .unwrap_or_else(|_| std::path::PathBuf::from(&notes_dir));
        match router.export(std::path::Path::new(&dir), force, &config_path, &notes_path) {
            Ok(()) => {
                eprintln!("Exported to {dir}");
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Export failed: {e}");
                std::process::exit(1);
            }
        }
    }

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
