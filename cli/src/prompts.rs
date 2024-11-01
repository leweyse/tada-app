#![deny(clippy::all)]

use std::collections::BTreeMap;
use std::ffi::OsString;

use anyhow::Context;
use inquire::{formatter::MultiOptionFormatter, Confirm, MultiSelect, Select, Text};

use super::utils::fs::Details;

pub fn select_template(options: BTreeMap<String, OsString>, selected_template: &mut Details) {
    let options_names = options.keys().cloned().collect::<Vec<String>>();

    let template_selected = Select::new("What template do you want to use?", options_names)
        .prompt()
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
    // let validator = |a: &[ListOption<&String>]| {
    //   let x = a.iter().any(|o| *o.value == "tailwindcss");
    //
    //   match x {
    //     true => Ok(Validation::Valid),
    //     false => Ok(Validation::Invalid("Remember to use tailwindcss".into())),
    //   }
    // };

    let formatter: MultiOptionFormatter<'_, String> = &|a| format!("{} addon(s)", a.len());

    let options_names = options.keys().cloned().collect::<Vec<String>>();

    let addons_selected = MultiSelect::new("Select the addons you want to use:", options_names)
        .with_formatter(formatter)
        .prompt()
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
    let name_provided = Text::new("Where do you want to create the project?")
        .with_placeholder("./my-project")
        .with_default(&default_name)
        .prompt()
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

pub fn try_installing_deps() -> Result<bool, anyhow::Error> {
    return Confirm::new("Do you want to install the dependencies?")
        .with_default(true)
        .prompt()
        .with_context(|| "No confirmation provided, exiting");
}
