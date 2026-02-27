use colored::Colorize;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::convert;

#[derive(Debug, PartialEq, Eq)]
pub enum EntryStatus {
    Ready,
    TargetExists,
}

#[derive(Debug)]
pub struct RenameEntry {
    pub source: PathBuf,
    pub target: PathBuf,
    pub status: EntryStatus,
}

pub struct RenamePlan {
    pub entries: Vec<RenameEntry>,
    pub skipped: usize,
}

impl RenamePlan {
    pub fn build(paths: &[String], separator: char, verbose: bool) -> Self {
        let mut entries = Vec::new();
        let mut skipped = 0;

        for path_str in paths {
            let source = PathBuf::from(path_str);

            if source.symlink_metadata().is_err() {
                eprintln!("{} {} (not found)", "skip:".yellow(), path_str);
                skipped += 1;
                continue;
            }

            let filename = match source.file_name().and_then(|f| f.to_str()) {
                Some(f) => f.to_string(),
                None => {
                    eprintln!("{} cannot extract filename: {}", "skip:".yellow(), path_str);
                    skipped += 1;
                    continue;
                }
            };

            let converted = convert::convert_filename(&filename, separator);

            if converted.is_empty() {
                eprintln!(
                    "{} conversion produced empty name: {}",
                    "skip:".yellow(),
                    path_str
                );
                skipped += 1;
                continue;
            }

            if converted == filename {
                if verbose {
                    eprintln!("{} {} (unchanged)", "skip:".dimmed(), path_str);
                }
                skipped += 1;
                continue;
            }

            let target = source.with_file_name(&converted);
            let status = if target.exists() || target.symlink_metadata().is_ok() {
                EntryStatus::TargetExists
            } else {
                EntryStatus::Ready
            };

            entries.push(RenameEntry {
                source,
                target,
                status,
            });
        }

        RenamePlan { entries, skipped }
    }

    /// Check if multiple sources would rename to the same target.
    /// Returns true if collisions were found.
    pub fn check_collisions(&self) -> bool {
        let mut target_map: HashMap<&PathBuf, Vec<&PathBuf>> = HashMap::new();
        for entry in &self.entries {
            target_map
                .entry(&entry.target)
                .or_default()
                .push(&entry.source);
        }
        let mut found = false;
        for (target, sources) in &target_map {
            if sources.len() > 1 {
                eprintln!(
                    "{} multiple files would rename to {}:",
                    "collision:".red().bold(),
                    target.display()
                );
                for s in sources {
                    eprintln!("  - {}", s.display());
                }
                found = true;
            }
        }
        found
    }

    pub fn display(&self, execute: bool) {
        for entry in &self.entries {
            let target_name = entry
                .target
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();

            match entry.status {
                EntryStatus::TargetExists => {
                    eprintln!(
                        "{} {} {} {}",
                        entry.source.display(),
                        "->".bold(),
                        target_name.red(),
                        "(already exists)".red(),
                    );
                }
                EntryStatus::Ready => {
                    println!(
                        "{} {} {} {}",
                        entry.source.display(),
                        "->".bold(),
                        target_name.green(),
                        if execute { "" } else { "(dry-run)" }
                    );
                }
            }
        }
    }

}
