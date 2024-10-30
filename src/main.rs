#[macro_use]
extern crate lazy_static;
use mlua::prelude::*;
use std::{fs, os::unix, path::Path, string};

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

lazy_static! {
    static ref PACKAGE_STORE: Mutex<HashMap<String, HashSet<String>>> = Mutex::new(HashMap::new());
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

    module.set("setup", setup_function)?;

    module.set("create_symlink", create_symlink)?;

    module.set("packages", add_packages)?;

    // symlink

    Ok(module)
}

fn main() -> Result<(), mlua::Error> {
    let lua = Lua::new();

    let lua_code =
        fs::read_to_string("./src/config.lua").expect("Should have been able to read the file");

    let globals = lua.globals();
    create_module(&lua, "luadec")?;

    println!("{:#?}", globals);
    lua.load(&lua_code).exec()?;

    Ok(())
}
