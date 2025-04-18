#![deny(clippy::all)]

use std::collections::BTreeMap;
use std::ffi::OsString;

use anyhow::Context;
use cliclack::{confirm, input, multiselect, select};

use super::utils::fs::Details;

pub fn select_template(options: BTreeMap<String, OsString>, selected_template: &mut Details) {
    let options_names = options
        .keys()
        .map(|x| (x.to_string(), x.clone(), ""))
        .collect::<Vec<_>>();

    let template_selected = select("What template do you want to use?")
        .items(options_names.as_slice())
        .interact()
        .with_context(|| "No template selected, exiting");

    match template_selected {
        Ok(selected) => {
            selected_template.name = selected.clone();
            selected_template.path = options
                .get(&selected)
                .with_context(|| "Error getting path")
                .unwrap()
                .to_os_string();
        }
        Err(e) => {
            println!("\n\n{:?}", e);
            std::process::exit(1);
        }
    }
}

pub fn select_addons(options: BTreeMap<String, OsString>, addons: &mut Vec<Details>) {
    let options_names = options
        .keys()
        .map(|x| (x.to_string(), x.clone(), ""))
        .collect::<Vec<_>>();

    let addons_selected = multiselect("Select the addons you want to use:")
        .items(options_names.as_slice())
        .required(false)
        .interact()
        .with_context(|| "No addons selected, exiting");

    match addons_selected {
        Ok(selected) => {
            for addon_name in selected {
                let path_to_selected_addon = options.get(&addon_name).unwrap();
                addons.push(Details {
                    name: addon_name.to_string(),
                    path: path_to_selected_addon.clone(),
                });
            }
        }
        Err(e) => {
            println!("\n{:?}", e);
            std::process::exit(1);
        }
    }
}

pub fn select_app_name(default_name: String, app_name: &mut String) {
    let name_provided = input("Where do you want to create the project?")
        .placeholder("./my-project")
        .default_input(&default_name)
        .interact()
        .with_context(|| "No project name provided, exiting");

    match name_provided {
        Ok(name) => {
            *app_name = name;
        }
        Err(e) => {
            println!("\n\n{:?}", e);
            std::process::exit(1);
        }
    }
}

pub fn try_installing_deps() -> bool {
    let value = confirm("Do you want to install the dependencies?")
        .initial_value(true)
        .interact()
        .with_context(|| "No confirmation provided, exiting")
        .unwrap();

    value
}
