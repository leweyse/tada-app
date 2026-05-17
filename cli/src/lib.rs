#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;
extern crate fs_extra;

mod prompts;
mod utils;

use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use relative_path::RelativePath;
use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use cliclack::{intro, outro, outro_cancel, spinner};
use console::style;

use utils::fs::{
    copy_addon_items, get_filtered_addons, get_items_in_template, get_templates, merge_map,
    read_json_file, tada_app_path, try_read_json_file, write_package_json, write_project_marker,
    AddonCopyOptions, Details, PackageJson, TadaJson, TadaProjectJson,
};
use utils::pm::install_dependencies;

use prompts::{prompt_app_path, prompt_install_deps, prompt_select_addons, prompt_select_template};

const IGNORE: [&str; 3] = ["node_modules", ".turbo", "dist"];

#[napi]
fn main() {
    ctrlc::set_handler(move || {}).expect("setting Ctrl-C handler");

    match cliclack::clear_screen().with_context(|| "Error clearing screen") {
        Ok(_) => {}
        Err(e) => eprintln!("Error clearing screen: {}", e),
    }

    match env::args().nth(2).as_deref() {
        None => run_create(),
        Some("add") => run_add(),
        Some(other) => {
            let _ = outro_cancel(format!(
                "Unknown command: {:?}. Usage: create-tada-app [add]",
                other
            ));
            std::process::exit(1);
        }
    }
}

fn run_create() {
    let _ = intro(style(" create-tada-app ").on_cyan().black());

    let tada_app_path = tada_app_path();

    let cwd: PathBuf = match env::current_dir() {
        Ok(path) => path,
        Err(_) => {
            let _ = outro_cancel("Error reading current directory");
            std::process::exit(1);
        }
    };

    let mut app_name = String::new();
    prompt_app_path(&mut app_name);

    let tada_templates_path = tada_app_path.join("templates");

    let mut templates: BTreeMap<String, OsString> = BTreeMap::new();
    get_templates(tada_templates_path.as_os_str(), &mut templates);

    if templates.is_empty() {
        let _ = outro_cancel("No templates found");
        std::process::exit(1);
    }

    let mut selected_template: Details = Details {
        name: "".to_string(),
        path: OsString::new(),
    };
    prompt_select_template(templates, &mut selected_template);

    let tada_addons_path = tada_app_path.join("addons");

    let mut addons: BTreeMap<String, OsString> = BTreeMap::new();
    get_filtered_addons(
        tada_addons_path.as_os_str(),
        selected_template.name.clone(),
        &mut addons,
    );

    let mut selected_addons: Vec<Details> = Vec::new();
    if !addons.is_empty() {
        prompt_select_addons(addons, &mut selected_addons);
    }

    let should_install_deps = prompt_install_deps();

    let new_app_path = RelativePath::new(&app_name).to_logical_path(cwd);

    if new_app_path.exists() {
        if let Some(parent) = new_app_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                let _ = outro_cancel(format!("Error creating directory: {:?}", e));
                std::process::exit(1);
            }
        }
    } else if let Err(e) = std::fs::create_dir_all(&new_app_path) {
        let _ = outro_cancel(format!("Error creating directory: {:?}", e));
        std::process::exit(1);
    }

    let items_to_ignore = IGNORE.map(|x| x.to_string()).to_vec();
    let items_in_template = get_items_in_template(&selected_template.path, items_to_ignore);
    let os_items_in_template = items_in_template
        .iter()
        .map(|x| Path::new(x).as_os_str())
        .collect::<Vec<_>>();

    let copy_template_spinner = spinner();
    copy_template_spinner.start("Copying template...");

    if let Err(e) = copy_items(
        &os_items_in_template,
        new_app_path.as_os_str(),
        &CopyOptions::new(),
    ) {
        let _ = outro_cancel(format!("Error copying template: {:?}", e));
        std::process::exit(1);
    }

    copy_template_spinner.stop("Template ready!");

    let project_package_json_path = new_app_path.join("package.json");
    let mut project_package_json: PackageJson =
        read_json_file(project_package_json_path.as_os_str());

    let mut installed: Vec<String> = Vec::new();
    if let Err(e) = apply_addons(
        &selected_addons,
        &new_app_path,
        &mut project_package_json,
        &mut installed,
    ) {
        let _ = outro_cancel(format!("Error applying addons: {:?}", e));
        std::process::exit(1);
    }

    project_package_json.name = match new_app_path.file_name() {
        Some(name) => name.to_str().unwrap().to_string(),
        None => selected_template.name.clone(),
    };

    if let Err(e) = write_package_json(&project_package_json_path, &project_package_json) {
        let _ = outro_cancel(format!("Error writing package.json: {:?}", e));
        std::process::exit(1);
    }

    if let Err(e) = write_project_marker(&new_app_path, &selected_template.name, &installed) {
        let _ = outro_cancel(format!("Error writing tada.json: {:?}", e));
        std::process::exit(1);
    }

    if should_install_deps {
        let install_deps_spinner = spinner();
        install_deps_spinner.start(format!(
            "Installing dependencies with {pnpm}...",
            pnpm = style("pnpm").magenta()
        ));

        match install_dependencies(new_app_path) {
            Ok(()) => install_deps_spinner.stop("Dependencies installed!"),
            Err(e) => install_deps_spinner.stop(format!("Failed to install dependencies: {}", e)),
        }
    }

    match outro(format!("{message} 🎉", message = style("ENJOY!").green())) {
        Ok(_) => {}
        Err(e) => eprintln!("Error printing outro: {}", e),
    }
}

fn run_add() {
    let _ = intro(style(" create-tada-app ").on_cyan().black());

    let tada_app_path = tada_app_path();

    let cwd: PathBuf = match env::current_dir() {
        Ok(path) => path,
        Err(_) => {
            let _ = outro_cancel("Error reading current directory");
            std::process::exit(1);
        }
    };

    let marker_path = cwd.join("tada.json");
    if !marker_path.exists() {
        let _ = outro_cancel("Not a tada-app project — run `create-tada-app` first");
        std::process::exit(1);
    }

    let mut project: TadaProjectJson = match try_read_json_file(marker_path.as_os_str()) {
        Ok(p) => p,
        Err(e) => {
            let _ = outro_cancel(format!("tada.json is malformed: {:?}", e));
            std::process::exit(1);
        }
    };

    let tada_addons_path = tada_app_path.join("addons");

    let mut available: BTreeMap<String, OsString> = BTreeMap::new();
    get_filtered_addons(
        tada_addons_path.as_os_str(),
        project.template.clone(),
        &mut available,
    );

    for installed_name in &project.addons {
        available.remove(installed_name);
    }

    if available.is_empty() {
        let _ = outro("All compatible addons are already installed");
        return;
    }

    let mut selected_addons: Vec<Details> = Vec::new();
    prompt_select_addons(available, &mut selected_addons);

    if selected_addons.is_empty() {
        let _ = outro("No addons selected");
        return;
    }

    let should_install_deps = prompt_install_deps();

    let project_package_json_path = cwd.join("package.json");
    if !project_package_json_path.exists() {
        let _ = outro_cancel("package.json not found in current directory");
        std::process::exit(1);
    }
    let mut project_package_json: PackageJson =
        read_json_file(project_package_json_path.as_os_str());

    let mut newly_installed: Vec<String> = Vec::new();
    if let Err(e) = apply_addons(
        &selected_addons,
        &cwd,
        &mut project_package_json,
        &mut newly_installed,
    ) {
        let _ = outro_cancel(format!("Error applying addons: {:?}", e));
        std::process::exit(1);
    }

    if let Err(e) = write_package_json(&project_package_json_path, &project_package_json) {
        let _ = outro_cancel(format!("Error writing package.json: {:?}", e));
        std::process::exit(1);
    }

    project.addons.extend(newly_installed);
    if let Err(e) = write_project_marker(&cwd, &project.template, &project.addons) {
        let _ = outro_cancel(format!("Error writing tada.json: {:?}", e));
        std::process::exit(1);
    }

    if should_install_deps {
        let install_deps_spinner = spinner();
        install_deps_spinner.start(format!(
            "Installing dependencies with {pnpm}...",
            pnpm = style("pnpm").magenta()
        ));

        match install_dependencies(cwd) {
            Ok(()) => install_deps_spinner.stop("Dependencies installed!"),
            Err(e) => install_deps_spinner.stop(format!("Failed to install dependencies: {}", e)),
        }
    }

    match outro(format!("{message} 🎉", message = style("ENJOY!").green())) {
        Ok(_) => {}
        Err(e) => eprintln!("Error printing outro: {}", e),
    }
}

fn apply_addons(
    addons: &[Details],
    project_root: &Path,
    pkg: &mut PackageJson,
    installed: &mut Vec<String>,
) -> anyhow::Result<()> {
    if addons.is_empty() {
        return Ok(());
    }

    let copy_addons_spinner = spinner();
    copy_addons_spinner.start("Copying addons...");

    for addon in addons {
        let addon_path = Path::new(&addon.path);

        let addon_tada_json_path = addon_path.join("tada.json");
        let addon_package_json_path = addon_path.join("package.json");

        let addon_tada_json: TadaJson = read_json_file(addon_tada_json_path.as_os_str());
        let addon_package_json: PackageJson = read_json_file(addon_package_json_path.as_os_str());

        merge_map(&mut pkg.dependencies, addon_package_json.dependencies);
        merge_map(&mut pkg.devDependencies, addon_package_json.devDependencies);
        merge_map(&mut pkg.scripts, addon_package_json.scripts);

        for entry in &addon_tada_json.entries {
            let source = addon_path.join(&entry.input);
            let destination = project_root.join(&entry.output);

            let file_name = entry.file_name.as_deref().unwrap_or_else(|| {
                Path::new(&entry.input)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .expect("addon entry has no valid file name")
            });

            if let Err(e) = copy_addon_items(
                &[source.as_os_str()],
                destination.as_os_str(),
                &AddonCopyOptions {
                    mode: &entry.mode,
                    file_name,
                },
            ) {
                return Err(anyhow!(
                    "Error copying addon {:?}, from: {:?}, to: {:?}: {:?}",
                    addon.name,
                    source,
                    destination,
                    e
                ));
            }
        }

        installed.push(addon.name.clone());
    }

    copy_addons_spinner.stop("Addons ready!");
    Ok(())
}
