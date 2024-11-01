use std::{path::PathBuf, process::Command};

fn pnpm() -> Command {
    #[cfg(windows)]
    const PNPM: &str = "pnpm.CMD";
    #[cfg(not(windows))]
    const PNPM: &str = "pnpm";

    Command::new(PNPM)
}

fn yarn() -> Command {
    #[cfg(windows)]
    const YARN: &str = "yarn.CMD";
    #[cfg(not(windows))]
    const YARN: &str = "yarn";

    Command::new(YARN)
}

fn npm() -> Command {
    #[cfg(windows)]
    const NPM: &str = "npm.CMD";
    #[cfg(not(windows))]
    const NPM: &str = "npm";

    Command::new(NPM)
}

pub fn install_dependencies(pm: &str, path: PathBuf) -> bool {
    let mut command = match pm {
        "pnpm" => pnpm(),
        "yarn" => yarn(),
        "npm" => npm(),
        _ => panic!("Invalid package manager"),
    };

    match command.current_dir(path).arg("install").status() {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}
