mod library;

#[macro_use]
extern crate lazy_static;
use directories::ProjectDirs;
use mlua::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use std::vec;
use std::{
    fs::{self, File},
    io::{self, Write},
    os::unix,
    path::Path,
    string,
};

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use library::*;

// I guess ill just throw the data here and do rust stuff after lua i done.
lazy_static! {
    static ref PACKAGE_STORE: Mutex<HashMap<String, HashSet<String>>> = Mutex::new(HashMap::new());
    static ref MANAGERS: Mutex<Vec<Manager>> = Mutex::new(Vec::new());
}

#[derive(Debug, Clone)]
struct Manager {
    name: String,
    add: String,
    remove: String,
    sync: String,
    upgrade: String,
}

fn lua_table_to_hashmap(table: mlua::Table) -> mlua::Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for pair in table.pairs::<String, String>() {
        let (key, value) = pair?;
        map.insert(key, value);
    }
    Ok(map)
}

pub fn create_module<'a>(lua: &'a Lua, name: &'a str) -> Result<mlua::Table<'a>, mlua::Error> {
    let mut luaa = lua.clone();
    let globals = lua.globals();
    let package: mlua::Table = globals.get("package")?;
    let loaded: mlua::Table = package.get("loaded")?;

    let module = lua.create_table()?;

    loaded.set(name, module.clone())?;
    let setup_function = lua.create_function(|_, ()| {
        println!("Hello, luadec!");
        Ok(())
    })?;

    // Symlinking
    let create_symlink = lua.create_function(|_, (original, destination): (String, String)| {
        {
            if Path::new(&original).exists() {
            } else {
                println!("symlink orgin {:#?} does not exist", &original);
            }
        }

        //let first_char_org = original.chars().next().unwrap();
        //let first_char_dest = destination.chars().next().unwrap();
        //println!(
        //    "original: {:#?}\nlink: {:#?}",
        //    first_char_org, first_char_dest
        //);

        unix::fs::symlink(original, destination);
        //println!("Hello, luadec!");
        Ok(())
    })?;

    let add_packages = lua.create_function(|_, (manager, packages): (String, mlua::Table)| {
        let mut store = PACKAGE_STORE.lock().unwrap();
        let entry = store.entry(manager.clone()).or_default();

        for package in packages.pairs::<LuaString, LuaString>().flatten() {
            let (_, package) = package;
            let package_str = package
                .to_str()
                .expect("Failed to convert package to string");

            if !entry.insert(package_str.to_string()) {
                panic!("Duplicate package detected: {}", package_str);
            }
        }

        // Print the current package store after installation
        println!("Current Package Store: {:?}", *store);

        Ok(())
    })?;

    let add_manager = lua.create_function(|_, manager_table: mlua::Table| {
        let name: String = manager_table.get("name")?;
        let add: String = manager_table.get("add")?;
        let remove: String = manager_table.get("remove")?;
        let sync: String = manager_table.get("sync")?;
        let upgrade: String = manager_table.get("upgrade")?;

        let package_manager = Manager {
            name,
            add,
            remove,
            sync,
            upgrade,
        };

        {
            let mut managers = MANAGERS.lock().unwrap();
            managers.push(package_manager);
        }

        Ok(())
    })?;

    module.set("setup", setup_function)?;

    module.set("create_symlink", create_symlink)?;

    module.set("packages", add_packages)?;
    module.set("add_manager", add_manager)?;

    Ok(module)
}

fn check_managers() -> (Vec<String>, Vec<String>) {
    let defined_managers = MANAGERS.lock().unwrap();
    let defined_set: HashSet<String> = defined_managers
        .iter()
        .map(|manager| manager.name.clone())
        .collect();

    let used_managers = PACKAGE_STORE.lock().unwrap();
    let used_set: HashSet<String> = used_managers
        .iter()
        .map(|manager| manager.0.clone())
        .collect();

    let common: Vec<String> = defined_set.intersection(&used_set).cloned().collect();

    // Find unique elements in vec2
    let undefined_managers: Vec<String> = used_set.difference(&defined_set).cloned().collect();

    return (common, undefined_managers);
}

//fn get_packages_from_manager(manager_name: String) -> Vec<String> {
//    let manager = PACKAGE_STORE.lock().unwrap();
//    let used_set: HashSet<String> = manager.into_keys;
//    let x: Vec<String> = Vec::new();
//    x
//}
fn get_packages_from_manager(manager_name: &str) -> Vec<String> {
    let package_store = PACKAGE_STORE.lock().unwrap();
    return package_store
        .get(manager_name)
        .into_iter()
        .flat_map(|set| set.iter().cloned())
        .collect();
}

fn get_manager_from_name(manager_name: String) -> Vec<Manager> {
    let managers = MANAGERS.lock().unwrap();

    managers
        .iter()
        .filter(|&manager| manager.name == manager_name)
        .cloned()
        .collect()
}

fn get_install_commands(manager: Manager, package_names: Vec<String>) {
    let add_command = manager.add.clone();

    package_names.iter().for_each(|package| {
        let command = add_command.replace("#:?", package);
        if !run_command(command.as_str()) {
            println!("FUCK");
        }
    });
}

fn get_config_path() -> PathBuf {
    if cfg!(debug_assertions) {
        let path = Path::new("./src/config.lua");
        path.to_path_buf()
    } else {
        ProjectDirs::from("com", "ToastedProducts", "Luadec")
            .map(|dirs| dirs.config_dir().join("config.lua"))
            .inspect(|path| {
                if !path.exists() {
                    fs::create_dir_all(path.parent().unwrap()).ok();
                    File::create(path).ok();
                }
            })
            .unwrap_or_else(|| panic!("Could not determine configuration directory"))
    }
}

fn main() -> Result<(), mlua::Error> {
    let lua = Lua::new();
    let config_path = get_config_path();
    println!("{:?}", config_path);
    let lua_code = fs::read_to_string(config_path).expect("Should have been able to read the file");

    let globals = lua.globals();
    create_module(&lua, "luadec")?;

    //println!("{:#?}", globals);
    lua.load(&lua_code).exec()?;

    // Print all packages stored in PACKAGE_STORE
    //let package_store = PACKAGE_STORE.lock().unwrap();
    //println!("Current Package Store: {:?}", *package_store);

    //let managers = MANAGERS.lock().unwrap();
    let (defined_managers, undefined_managers) = check_managers();

    if !undefined_managers.is_empty() {
        println!("DID NOT DEFINE MANAGER: {:#?}", undefined_managers);
    }

    for manager_name in defined_managers {
        let packages = get_packages_from_manager(manager_name.as_str());
        let managers = get_manager_from_name(manager_name);
        let manager = managers
            .first()
            .unwrap_or(&Manager {
                name: "No manager found".to_string(),
                add: "No manager found".to_string(),
                remove: "No manager found".to_string(),
                sync: "No manager found".to_string(),
                upgrade: "No manager found".to_string(),
            })
            .clone();

        get_install_commands(manager, packages);
        //packages
        //    .iter()
        //    .for_each(|package| get_install_commands(manager.clone(), package));
    }

    Ok(())
}
