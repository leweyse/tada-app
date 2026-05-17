use std::{path::PathBuf, process::Command};

use anyhow::{anyhow, Result};

fn pnpm() -> Command {
    #[cfg(windows)]
    const PNPM: &str = "pnpm.CMD";
    #[cfg(not(windows))]
    const PNPM: &str = "pnpm";

    Command::new(PNPM)
}

pub fn install_dependencies(path: PathBuf) -> Result<()> {
    let mut command = pnpm();

    let status = match command.current_dir(path).arg("install").status() {
        Ok(s) => s,
        Err(e) => return Err(anyhow!("Failed to run pnpm: {}", e)),
    };

    if !status.success() {
        return Err(anyhow!("pnpm install exited with status: {}", status));
    }

    Ok(())
}
