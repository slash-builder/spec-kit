//! `spec-kit` CLI — thin wrapper over the library.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "spec-kit", version, about = "Lockamy Studios spec tooling")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new spec skeleton.
    New {
        /// Spec name (e.g. "quickring-protocol").
        name: String,
    },
    /// Verify a spec markdown file.
    Verify { path: std::path::PathBuf },
    /// Classify changes between two spec versions.
    Diff {
        a: std::path::PathBuf,
        b: std::path::PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::New { name } => {
            println!("{}", spec_kit::template::skeleton(&name));
        }
        Commands::Verify { path } => {
            let contents = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("error reading {}: {e}", path.display());
                    std::process::exit(2);
                }
            };
            match spec_kit::template::verify(&contents) {
                Ok(()) => println!("{}: OK", path.display()),
                Err(e) => {
                    eprintln!("{}: {e}", path.display());
                    std::process::exit(1);
                }
            }
        }
        Commands::Diff { a, b } => {
            let a_contents = match std::fs::read_to_string(&a) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("error reading {}: {e}", a.display());
                    std::process::exit(2);
                }
            };
            let b_contents = match std::fs::read_to_string(&b) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("error reading {}: {e}", b.display());
                    std::process::exit(2);
                }
            };
            let kind = spec_kit::diff::compare(&a_contents, &b_contents);
            println!("{kind:?}");
        }
    }
}
