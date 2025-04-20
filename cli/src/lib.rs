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
use std::fs::File;
use std::path::Path;

use anyhow::Context;
use cliclack::{intro, outro, spinner, ProgressBar};

use utils::fs::{
    copy_addon_items, get_filtered_addons, get_items_in_template, get_templates, read_json_file,
    Details, PackageJson, TadaJson,
};
use utils::pm::install_dependencies;

use prompts::{select_addons, select_app_name, select_template, try_installing_deps};

const ENV_VAR: &str = "TADA_APP";
const IGNORE: [&str; 3] = ["node_modules", ".turbo", "dist"];

pub fn start_spinner(message: &str) -> ProgressBar {
    let spinner = spinner();

    spinner.start(message);

    spinner
}

#[napi]
fn main() {
    let tada_app_path: OsString;

    match env::var(ENV_VAR) {
        Ok(path) => tada_app_path = OsString::from(path),
        Err(_) => std::process::exit(1),
    }

    let tada_templates_path = Path::new(&tada_app_path.clone()).join("templates");

    let mut templates: BTreeMap<String, OsString> = BTreeMap::new();
    get_templates(tada_templates_path.as_os_str(), &mut templates);

    if templates.is_empty() {
        println!("No templates found, exiting");
        std::process::exit(1);
    }

    let _ = intro("create-tada-app").with_context(|| "Error printing intro");

    let mut selected_template: Details = Details {
        name: "".to_string(),
        path: OsString::new(),
    };
    select_template(templates, &mut selected_template);

    let tada_addons_path = Path::new(&tada_app_path.clone()).join("addons");

    let mut addons: BTreeMap<String, OsString> = BTreeMap::new();
    get_filtered_addons(
        tada_addons_path.as_os_str(),
        selected_template.name.clone(),
        &mut addons,
    );

    let mut selected_addons: Vec<Details> = Vec::new();
    if !addons.is_empty() {
        select_addons(addons, &mut selected_addons);
    }

    let mut app_name = String::new();
    select_app_name(selected_template.name.clone(), &mut app_name);

    let should_install_deps = try_installing_deps();

    let cwd = env::current_dir()
        .with_context(|| "Error reading current directory")
        .unwrap();

    let new_app_path = RelativePath::new(&app_name).to_logical_path(cwd);

    if new_app_path.exists() {
        if let Some(parent) = new_app_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| "Error creating directory")
                .unwrap();
        }
    } else {
        std::fs::create_dir_all(&new_app_path)
            .with_context(|| "Error creating directory")
            .unwrap();
    }

    let items_to_ignore = IGNORE.map(|x| x.to_string()).to_vec();
    let items_in_template = get_items_in_template(&selected_template.path, items_to_ignore);
    let os_items_in_template = items_in_template
        .iter()
        .map(|x| Path::new(x).as_os_str())
        .collect::<Vec<_>>();

    let copy_template_spinner = start_spinner("Copying template...");

    let template_copied = copy_items(
        &os_items_in_template,
        new_app_path.as_os_str(),
        &CopyOptions::new(),
    )
    .with_context(|| "Error copying template");

    match template_copied {
        Ok(_) => copy_template_spinner.stop("Template ready!"),
        Err(e) => println!("{:?}", e),
    }

    let project_package_json_path = Path::new(&new_app_path).join("package.json");
    let mut project_package_json: PackageJson =
        read_json_file(project_package_json_path.as_os_str());

    let mut dependencies: BTreeMap<String, String> = BTreeMap::new();
    for (key, value) in project_package_json.dependencies.unwrap() {
        dependencies.insert(key, value);
    }

    let mut dev_dependencies: BTreeMap<String, String> = BTreeMap::new();
    for (key, value) in project_package_json.devDependencies.unwrap() {
        dev_dependencies.insert(key, value);
    }

    let mut scripts: BTreeMap<String, String> = BTreeMap::new();
    if let Some(scripts_map) = project_package_json.scripts {
        for (key, value) in scripts_map {
            scripts.insert(key, value);
        }
    }

    if !selected_addons.is_empty() {
        let copy_addons_spinner = start_spinner("Copying addons...");

        for addon in &selected_addons {
            let addon_path = Path::new(&addon.path);

            let addon_tada_json_path = Path::new(addon_path).join("tada.json");
            let addon_package_json_path = Path::new(addon_path).join("package.json");

            let addon_tada_json: TadaJson = read_json_file(addon_tada_json_path.as_os_str());
            let addon_package_json: PackageJson =
                read_json_file(addon_package_json_path.as_os_str());

            if let Some(dependencies_map) = addon_package_json.dependencies {
                for (key, value) in dependencies_map {
                    dependencies.insert(key, value);
                }
            }

            if let Some(dev_dependencies_map) = addon_package_json.devDependencies {
                for (key, value) in dev_dependencies_map {
                    dev_dependencies.insert(key, value);
                }
            }

            if let Some(scripts_map) = addon_package_json.scripts {
                for (key, value) in scripts_map {
                    scripts.insert(key, value);
                }
            }

            for addon_entry in &addon_tada_json.entries {
                let addon_entry_source =
                    Path::new(&addon_path).join(OsString::from(&addon_entry.input).as_os_str());
                let addon_entry_destination =
                    new_app_path.join(OsString::from(&addon_entry.output).as_os_str());

                let addon_entry_os_source = addon_entry_source.as_os_str();
                let addon_entry_os_destination = addon_entry_destination.as_os_str();

                let addon_copied = copy_addon_items(
                    &[addon_entry_os_source],
                    addon_entry_os_destination,
                    &addon_entry.mode,
                )
                .with_context(|| {
                    format!(
                        "Error copying addon: {:?}, from: {:?}, to: {:?}",
                        addon.name, addon_entry_os_source, addon_entry_os_destination
                    )
                });

                match addon_copied {
                    Ok(_) => (),
                    Err(e) => {
                        println!("{:?}", e);
                        std::process::exit(1);
                    }
                }
            }
        }

        copy_addons_spinner.stop("Addons ready!");
    }

    project_package_json.name = match new_app_path.file_name() {
        Some(name) => name.to_str().unwrap().to_string(),
        None => selected_template.name.clone(),
    };
    project_package_json.scripts = Some(scripts);
    project_package_json.dependencies = Some(dependencies);
    project_package_json.devDependencies = Some(dev_dependencies);

    let new_package_json_string = serde_json::to_string_pretty(&project_package_json)
        .with_context(|| "Error serializing package.json")
        .unwrap();

    let mut new_package_json_file = File::create(project_package_json_path)
        .with_context(|| "Error creating package.json file")
        .unwrap();

    std::io::Write::write(
        &mut new_package_json_file,
        new_package_json_string.as_bytes(),
    )
    .with_context(|| "Error writing package.json file")
    .unwrap();

    if should_install_deps {
        let install_deps_spinner = start_spinner("Installing dependencies...");

        if install_dependencies("pnpm", new_app_path) {
            install_deps_spinner.stop("Dependencies installed!");
        }
    }

    let _ = outro("ENJOY! 🎉").with_context(|| "Error printing outro");
}
