use clap::Parser;
use std::path::PathBuf;

/// Rename Japanese filenames to romaji
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Files to rename. Reads from stdin if omitted and input is piped.
    pub files: Vec<PathBuf>,

    /// Execute renames (default is dry-run preview)
    #[arg(short = 'y', long = "yes")]
    pub yes: bool,

    /// Confirm each rename interactively
    #[arg(short, long)]
    pub interactive: bool,

    /// Show each operation
    #[arg(short, long)]
    pub verbose: bool,

    /// Character to replace spaces with
    #[arg(short, long, default_value = "_")]
    pub separator: char,
}
