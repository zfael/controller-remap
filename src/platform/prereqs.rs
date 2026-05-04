use std::process::Command;

use anyhow::{bail, Context, Result};

pub fn check_prerequisites() -> Result<()> {
    let output = Command::new("sc")
        .args(["query", "HidHide"])
        .output()
        .context("failed to query HidHide service")?;

    if !output.status.success() {
        bail!(
            "HidHide service not found; install HidHide and whitelist this executable before running"
        );
    }

    Ok(())
}
