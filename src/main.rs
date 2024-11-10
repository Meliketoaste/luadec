mod library;

#[macro_use]
extern crate lazy_static;
use directories::ProjectDirs;
use mlua::prelude::*;
use std::io::Read;
use std::path::PathBuf;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    os::unix,
    path::Path,
    string,
};

use serde_json; // Add serde_json for JSON serialization

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

fn get_current_store() -> HashMap<String, HashSet<String>> {
    let package_store_file: PathBuf = get_config_path().with_file_name("package_store.json");

    let mut package_store: HashMap<String, HashSet<String>> = HashMap::new();

    if Path::new(&package_store_file).exists() {
        let mut file = File::open(&package_store_file).expect("Unable to open package store file");

        let mut data = String::new();
        file.read_to_string(&mut data)
            .expect("Failed to read package store data");

        package_store = serde_json::from_str(&data).unwrap_or_else(|_| HashMap::new());
        //println!("woahh{:#?}", data);
    } else {
        File::create(package_store_file).expect("failed  to create package_store_file");
    }
    //println!("{:#?}", package_store);
    package_store
}

fn get_new_packages(
    old_packages: &HashMap<String, HashSet<String>>,
    new_packages: &HashMap<String, HashSet<String>>,
) -> HashMap<String, HashSet<String>> {
    let mut new_additions: HashMap<String, HashSet<String>> = HashMap::new();

    for (manager, new_package_set) in new_packages.iter() {
        let hash_set = HashSet::new();
        let old_package_set = old_packages.get(manager).unwrap_or(&hash_set);

        let new_entries: HashSet<String> = new_package_set
            .difference(old_package_set)
            .cloned()
            .collect();

        if !new_entries.is_empty() {
            new_additions.insert(manager.clone(), new_entries);
        }
    }

    new_additions
}

fn add_new_packages(new_additions: HashMap<String, HashSet<String>>) -> io::Result<()> {
    let mut current_store = get_current_store();

    for (manager, new_packages) in new_additions {
        let entry = current_store.entry(manager).or_default();
        for package in new_packages {
            entry.insert(package);
        }
    }

    let json_data = serde_json::to_string(&current_store).expect("Failed to serialize to JSON");

    let package_store_file = get_config_path().with_file_name("package_store.json");

    let mut file = File::create(&package_store_file)?;
    file.write_all(json_data.as_bytes())?;

    Ok(())
}
fn lua_table_to_hashmap(table: mlua::Table) -> mlua::Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for pair in table.pairs::<String, String>() {
        let (key, value) = pair?;
        map.insert(key, value);
    }
    Ok(map)
}

fn create_module<'a>(lua: &'a Lua, name: &'a str) -> Result<mlua::Table<'a>, mlua::Error> {
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

    let manage_file = lua.create_function(|_, (dest, config): (String, mlua::Table)| {
        let mut map: HashMap<String, String> = HashMap::new();
        let source: String = config.get("content")?;

        let vars: LuaTable = config.get("vars")?;
        // if vars.is_empty() {
        //     println!("WOAHHH");
        // }
        for pair in vars.clone().pairs::<String, String>() {
            let (key, value) = pair?;
            map.insert(key, value);
        }

        println!("dest: {:#?}", dest);
        println!("source: {:#?}", source);
        //println!("vars is: {:#?}", vars);
        println!("map is: {:#?}", map);

        if Path::new(&dest).exists() {
            if Path::new(&source).exists() {
                if let Err(e) = unix::fs::symlink(&source, &dest) {
                    eprintln!("Failed to create symlink: {}", e);
                } else {
                    println!("Symlink created from {} to {}", source, dest);
                }
            } else {
                let mut content = source.clone();

                for (key, value) in &map {
                    content = content.replace(&format!("${{{}}}", key), value);
                }

                // Overwrite the existing destination file with new content
                if let Err(e) = fs::write(&dest, &content) {
                    eprintln!("Failed to write content to {}: {}", dest, e);
                } else {
                    println!("Content written to {}", dest);
                }
            }
        } else if Path::new(&source).exists() {
            if let Err(e) = unix::fs::symlink(&source, &dest) {
                eprintln!("Failed to create symlink: {}", e);
            } else {
                println!("Symlink created from {} to {}", source, dest);
            }
        } else {
            let mut content = source.clone();

            for (key, value) in &map {
                content = content.replace(&format!("${{{}}}", key), value);
            }

            if let Err(e) = fs::write(&dest, &content) {
                eprintln!("Failed to write content to {}: {}", dest, e);
            } else {
                println!("Content written to {}", dest);
            }
        }

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
        //println!("Current Package Store: {:?}", *store);

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

    //module.set("create_symlink", create_symlink)?;

    module.set("packages", add_packages)?;
    module.set("add_manager", add_manager)?;
    module.set("file", manage_file)?;

    Ok(module)
}

fn check_managers() -> HashMap<String, HashSet<String>> {
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

    let common: HashSet<String> = defined_set.intersection(&used_set).cloned().collect();

    let undefined_managers: HashSet<String> = used_set.difference(&defined_set).cloned().collect();

    let mut modified_manager_map: HashMap<String, HashSet<String>> = HashMap::new();

    for manager in common {
        if let Some(packages) = used_managers.get(&manager) {
            modified_manager_map.insert(manager, packages.clone());
        }
    }

    for manager in undefined_managers {
        modified_manager_map.insert(manager, HashSet::new()); // Empty set for undefined managers
    }

    modified_manager_map
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
    let old_store = get_current_store();
    let lua = Lua::new();
    let config_path = get_config_path();
    //println!("{:?}", config_path);
    let lua_code = fs::read_to_string(config_path).expect("Should have been able to read the file");

    //let globals = lua.globals();
    create_module(&lua, "luadec")?;

    lua.load(&lua_code).exec()?;

    let modified_manager_map = check_managers();
    let new_additions = get_new_packages(&old_store, &modified_manager_map);

    if let Err(e) = add_new_packages(new_additions.clone()) {
        eprintln!("Error adding new packages: {}", e);
    }

    for (manager, packages) in new_additions {
        println!("Manager: {}", manager);
        println!("New packages: {:?}", packages);

        let managers = get_manager_from_name(manager.clone());
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

        let packages_vec: Vec<String> = packages.into_iter().collect();
        get_install_commands(manager, packages_vec);
    }

    Ok(())
}
