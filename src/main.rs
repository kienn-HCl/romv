mod cli;
mod convert;
mod plan;
mod rename;

use anyhow::{Result, bail};
use clap::Parser;
use colored::Colorize;
use plan::EntryStatus;
use std::fs::File;
use std::io::{self, BufRead, BufReader, IsTerminal, Write};
use std::path::PathBuf;

fn main() -> Result<()> {
    let args = cli::Args::parse();

    match args.separator {
        '/' | '\0' => bail!("invalid separator: cannot use '/' or null character"),
        '.' => bail!("invalid separator: '.' would break file extension parsing"),
        _ => {}
    }

    let paths: Vec<PathBuf> = if !args.files.is_empty() {
        args.files
    } else if !io::stdin().is_terminal() {
        io::stdin()
            .lock()
            .lines()
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|l| !l.is_empty())
            .map(PathBuf::from)
            .collect()
    } else {
        cli::Args::parse_from(["romv", "--help"]);
        unreachable!();
    };

    let plan = plan::RenamePlan::build(&paths, args.separator, args.verbose);

    if plan.entries.is_empty() {
        if plan.skipped > 0 {
            eprintln!("Nothing to rename ({} skipped).", plan.skipped);
        } else {
            eprintln!("Nothing to rename.");
        }
        return Ok(());
    }

    if plan.check_collisions() {
        bail!("Aborting due to collisions. No files were renamed.");
    }

    let execute = args.yes || args.interactive;
    plan.display(execute);

    if !execute {
        eprintln!(
            "\n{}",
            "Dry-run complete. Use -y to execute or -i for interactive mode.".dimmed()
        );
        return Ok(());
    }

    let mut renamed = 0;
    let mut errors = 0;

    // Read interactive confirmations from /dev/tty so that piped stdin
    // (e.g. `ls | romv -i`) does not conflict with user input.
    let mut tty_reader = if args.interactive {
        Some(BufReader::new(
            File::open("/dev/tty").map_err(|e| anyhow::anyhow!("cannot open /dev/tty: {e}"))?,
        ))
    } else {
        None
    };

    for entry in &plan.entries {
        if entry.status != EntryStatus::Ready {
            errors += 1;
            continue;
        }

        if let Some(ref mut tty) = tty_reader {
            eprint!(
                "Rename {} -> {}? [y/N] ",
                entry.source.display(),
                entry
                    .target
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            );
            io::stderr().flush()?;
            let mut input = String::new();
            tty.read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                if args.verbose {
                    eprintln!(
                        "{} {} (user declined)",
                        "skip:".yellow(),
                        entry.source.display()
                    );
                }
                continue;
            }
        }

        match rename::safe_rename(&entry.source, &entry.target) {
            Ok(()) => {
                renamed += 1;
            }
            Err(e) => {
                eprintln!(
                    "{} {} -> {}: {}",
                    "error:".red().bold(),
                    entry.source.display(),
                    entry.target.display(),
                    e
                );
                errors += 1;
            }
        }
    }

    if args.verbose || errors > 0 {
        eprintln!(
            "Done: {renamed} renamed, {errors} errors, {} skipped.",
            plan.skipped
        );
    }

    if errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}
