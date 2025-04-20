#![deny(clippy::all)]

use fs_extra::dir;
use fs_extra::dir::{ls, DirEntryAttr, DirEntryValue};
use fs_extra::error::{Error, ErrorKind, Result};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::fmt::Debug;
use std::fs;
use std::io::{BufReader, Read, Seek, Write};
use std::path::Path;
use std::u64;

use anyhow::Context;
use diffy::{apply, create_patch};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum AddonFileCopyType {
    /// Sets the option true for overwrite existing files.
    #[serde(rename = "overwrite")]
    Overwrite,
    /// Sets the option true for prepending files.
    #[serde(rename = "prepend")]
    Prepend,
    /// Sets the option true for appending files.
    #[serde(rename = "append")]
    Append,
    /// Sets the option true for skipping existing files.
    #[serde(rename = "skip_if_exists")]
    SkipIfExists,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    pub input: String,
    pub output: String,

    pub mode: Option<AddonFileCopyType>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct TadaJson {
    templates: Vec<String>,
    pub entries: Vec<Entry>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct PackageJson {
    pub name: String,

    #[serde(flatten)]
    other: HashMap<String, serde_json::Value>,

    pub scripts: Option<BTreeMap<String, String>>,

    pub dependencies: Option<BTreeMap<String, String>>,
    pub devDependencies: Option<BTreeMap<String, String>>,
}

#[derive(Debug)]
pub struct Details {
    pub name: String,
    pub path: OsString,
}

pub fn read_json_file<T>(path: &OsStr) -> T
where
    T: DeserializeOwned,
{
    let file = fs::File::open(path)
        .with_context(|| format!("Error reading file: {}", path.to_str().unwrap()))
        .unwrap();

    let reader = BufReader::new(file);

    let package_json: T = serde_json::from_reader(reader)
        .with_context(|| "Error parsing JSON")
        .unwrap();

    package_json
}

pub fn get_templates(path: &OsStr, templates: &mut BTreeMap<String, OsString>) {
    let path_to = Path::new(&path);

    let mut config = HashSet::new();
    config.insert(DirEntryAttr::Name);
    config.insert(DirEntryAttr::Path);

    let templates_dir = ls(path_to, &config)
        .with_context(|| format!("Error reading templates directory: {:?}", path_to))
        .unwrap();

    for item in templates_dir.items {
        let template_path = item
            .get(&DirEntryAttr::Path)
            .with_context(|| "Error reading path")
            .unwrap();

        let template_name = item
            .get(&DirEntryAttr::Name)
            .with_context(|| "Error reading name")
            .unwrap();

        if let DirEntryValue::String(path) = template_path {
            if let DirEntryValue::String(name) = template_name {
                templates.insert(name.to_string(), OsString::from(path));
            }
        }
    }
}

pub fn get_filtered_addons(
    path: &OsStr,
    template_name: String,
    addons: &mut BTreeMap<String, OsString>,
) {
    let mut config = HashSet::new();
    config.insert(DirEntryAttr::Name);
    config.insert(DirEntryAttr::Path);

    let dirs = ls(path, &config)
        .with_context(|| format!("Error reading addons directory: {:?}", path))
        .unwrap();

    for item in dirs.items {
        let addon_path = item
            .get(&DirEntryAttr::Path)
            .with_context(|| "Addons: Error reading path")
            .unwrap();

        let addon_name = item
            .get(&DirEntryAttr::Name)
            .with_context(|| "Addons: Error reading name")
            .unwrap();

        if let DirEntryValue::String(path) = addon_path {
            if let DirEntryValue::String(name) = addon_name {
                let tada_json_path = Path::new(&path).join("tada.json");

                let tada_json: TadaJson = read_json_file(tada_json_path.as_os_str());

                if tada_json.templates.contains(&"all".to_string())
                    || tada_json.templates.contains(&template_name)
                {
                    addons.insert(name.to_string(), OsString::from(path));
                }
            }
        }
    }
}

pub fn get_items_in_template(path: &OsStr, ignore: Vec<String>) -> Vec<OsString> {
    let path_to = Path::new(&path);

    let mut config = HashSet::new();
    config.insert(DirEntryAttr::Name);
    config.insert(DirEntryAttr::Path);

    let mut items: Vec<OsString> = Vec::new();

    let dirs = ls(path_to, &config)
        .with_context(|| format!("Error reading addons directory: {:?}", path_to))
        .unwrap();

    for item in dirs.items {
        let item_path = item
            .get(&DirEntryAttr::Path)
            .with_context(|| "Addons: Error reading path")
            .unwrap();

        let item_name = item
            .get(&DirEntryAttr::Name)
            .with_context(|| "Addons: Error reading name")
            .unwrap();

        if let DirEntryValue::String(path) = item_path {
            if let DirEntryValue::String(name) = item_name {
                if ignore.contains(name) {
                    continue;
                }

                items.push(OsString::from(path));
            }
        }
    }

    items
}

pub fn copy_addon_file<P, Q>(from: P, to: Q, mode: &Option<AddonFileCopyType>) -> Result<u64>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let from = from.as_ref();
    if !from.exists() {
        if let Some(msg) = from.to_str() {
            let msg = format!("Path \"{}\" does not exist or you don't have access!", msg);
            return Err(Error::new(ErrorKind::NotFound, &msg));
        }
        return Err(Error::new(
            ErrorKind::NotFound,
            "Path does not exist or you don't have access!",
        ));
    }

    if !from.is_file() {
        if let Some(msg) = from.to_str() {
            let msg = format!("Path \"{}\" is not a file!", msg);
            return Err(Error::new(ErrorKind::InvalidFile, &msg));
        }
        return Err(Error::new(ErrorKind::InvalidFile, "Path is not a file!"));
    }

    let to = to.as_ref();

    if !to.exists() {
        match std::fs::copy(from, to) {
            Ok(_) => return Ok(0),
            Err(e) => {
                let msg = format!("Error creating file: {:?}", e);
                return Err(Error::new(ErrorKind::InvalidFile, &msg));
            }
        }
    }

    if let Some(mode) = mode {
        match mode {
            AddonFileCopyType::Overwrite => return Ok(std::fs::copy(from, to)?),
            AddonFileCopyType::SkipIfExists => return Ok(0),
            _ => {}
        }
    }

    let from_file = fs::File::open(from);
    let from_content = match from_file {
        Ok(file) => {
            let mut reader = BufReader::new(file);
            let mut content = String::new();
            match reader.read_to_string(&mut content) {
                Ok(_) => content,
                Err(e) => {
                    let msg = format!("Error reading file: {:?}", e);
                    return Err(Error::new(ErrorKind::InvalidFile, &msg));
                }
            }
        }
        Err(e) => {
            let msg = format!("Error reading file: {:?}", e);
            return Err(Error::new(ErrorKind::InvalidFile, &msg));
        }
    };

    let mut to_file = match fs::OpenOptions::new().read(true).write(true).open(to) {
        Ok(file) => file,
        Err(e) => {
            let msg = format!("Error creating file: {:?}", e);
            return Err(Error::new(ErrorKind::InvalidFile, &msg));
        }
    };

    let mut reader = BufReader::new(&to_file);
    let mut to_content = String::new();

    match reader.read_to_string(&mut to_content) {
        Ok(_) => {
            to_file.rewind().unwrap();
        }
        Err(e) => {
            let msg = format!("Error reading file: {:?}", e);
            return Err(Error::new(ErrorKind::InvalidFile, &msg));
        }
    };

    if let Some(mode) = mode {
        match mode {
            AddonFileCopyType::Prepend => {
                match to_file.write_all(format!("{}\n{}", from_content, to_content).as_bytes()) {
                    Ok(_) => {
                        return Ok(0);
                    }
                    Err(e) => {
                        let msg = format!("Error writing file: {:?}", e);
                        return Err(Error::new(ErrorKind::InvalidFile, &msg));
                    }
                };
            }
            AddonFileCopyType::Append => {
                match to_file.write_all(format!("{}\n{}", to_content, from_content).as_bytes()) {
                    Ok(_) => {
                        return Ok(0);
                    }
                    Err(e) => {
                        let msg = format!("Error writing file: {:?}", e);
                        return Err(Error::new(ErrorKind::InvalidFile, &msg));
                    }
                };
            }
            _ => {}
        }
    }

    // If the destination file is not empty, create a patch with it
    // as the original content and the new content as the modified
    // content.
    let patch = create_patch(&to_content, &from_content);
    let applied = match apply(&to_content, &patch) {
        Ok(result) => result,
        Err(e) => {
            let msg = format!("Error applying patch: {:?}", e);
            return Err(Error::new(ErrorKind::InvalidFile, &msg));
        }
    };

    match to_file.write_all(applied.as_bytes()) {
        Ok(_) => {
            return Ok(0);
        }
        Err(e) => {
            let msg = format!("Error writing file: {:?}", e);
            return Err(Error::new(ErrorKind::InvalidFile, &msg));
        }
    };
}

pub fn copy_addon_items<P, Q>(from: &[P], to: Q, mode: &Option<AddonFileCopyType>) -> Result<u64>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let mut result: u64 = 0;
    for item in from {
        let item = item.as_ref();
        if item.is_dir() {
            result += dir::copy(item, &to, &Default::default()).unwrap();
        } else if let Some(file_name) = item.file_name() {
            if let Some(file_name) = file_name.to_str() {
                result += copy_addon_file(item, to.as_ref().join(file_name), mode).unwrap();
            }
        } else {
            return Err(Error::new(ErrorKind::InvalidFileName, "Invalid file name"));
        }
    }

    Ok(result)
}
