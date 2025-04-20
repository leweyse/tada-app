#![deny(clippy::all)]

use std::collections::BTreeMap;
use std::ffi::OsString;

use anyhow::Context;
use cliclack::{confirm, input, multiselect, select};

use super::utils::fs::Details;

pub fn prompt_select_template(
    options: BTreeMap<String, OsString>,
    selected_template: &mut Details,
) {
    let options_names = options
        .keys()
        .map(|x| (x.to_string(), x.clone(), ""))
        .collect::<Vec<_>>();

    let template_selected = select("Choose a template")
        .items(options_names.as_slice())
        .interact()
        .with_context(|| "No template selected, exiting");

    match template_selected {
        Ok(selected) => {
            selected_template.name = selected.clone();
            selected_template.path = options
                .get(&selected)
                .with_context(|| "Unable to get template")
                .unwrap()
                .to_os_string();
        }
        Err(e) => {
            println!("\n\n{:?}", e);
            std::process::exit(1);
        }
    }
}

pub fn prompt_select_addons(options: BTreeMap<String, OsString>, addons: &mut Vec<Details>) {
    let options_names = options
        .keys()
        .map(|x| (x.to_string(), x.clone(), ""))
        .collect::<Vec<_>>();

    let addons_selected = multiselect("Choose the addons you want to use:")
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

pub fn prompt_app_path(app_name: &mut String) {
    let name_provided = input("What is the name of your project?")
        .placeholder("./my-project")
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

pub fn prompt_install_deps() -> bool {
    let value = confirm("Should we install the dependencies?")
        .initial_value(true)
        .interact()
        .with_context(|| "No confirmation provided, exiting")
        .unwrap();

    value
}
