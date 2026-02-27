use anyhow::{Result, bail};
use std::path::Path;

/// Rename `source` to `target` with no-clobber safety.
///
/// Returns an error if:
/// - `source` does not exist
/// - `target` already exists
pub fn safe_rename(source: &Path, target: &Path) -> Result<()> {
    if source.symlink_metadata().is_err() {
        bail!("source does not exist: {}", source.display());
    }
    if target.exists() || target.symlink_metadata().is_ok() {
        bail!(
            "target already exists: {} -> {}",
            source.display(),
            target.display()
        );
    }
    std::fs::rename(source, target)?;
    Ok(())
}
