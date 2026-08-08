use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};

use mlua::{Function, Lua};

use crate::scripting::loader::{WeaponDef, load_weapon};

pub struct WeaponRegistry(std::collections::HashMap<String, WeaponDef>);

impl WeaponRegistry {
    pub fn load_all(lua: &mlua::Lua, dir: &str) -> mlua::Result<Self> {
        let mut map = std::collections::HashMap::new();
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            let name = path.file_stem().unwrap().to_string_lossy().to_string();
            map.insert(name, load_weapon(lua, path.to_str().unwrap())?);
        }
        Ok(Self(map))
    }
}

pub fn register_commands(lua: &Lua) -> mlua::Result<Arc<Mutex<HashMap<String, Function>>>> {
    let commands: Arc<Mutex<HashMap<String, Function>>> = Arc::new(Mutex::new(HashMap::new()));

    let reg = commands.clone();
    let register_fn = lua.create_function(move |_, (name, func): (String, Function)| {
        reg.lock()
            .map_err(|e| mlua::Error::external(e.to_string()))?
            .insert(name, func);
        // reg.borrow_mut().insert(name, func);
        Ok(())
    })?;

    let table = lua.create_table()?;

    table.set("register", register_fn)?;
    lua.globals().set("commands", table)?;

    Ok(commands)
}

pub fn register_events(lua: &Lua) -> mlua::Result<()> {
    let listeners_table = lua.create_table()?;
    lua.set_named_registry_value("EVENT_LISTENERS", listeners_table)?;

    let events_table = lua.create_table()?;

    let on_fn = lua.create_function(|lua, (event_name, func): (String, Function)| {
        let listeners: mlua::Table = lua.named_registry_value("EVENT_LISTENERS")?;

        let list: mlua::Table = match listeners.get(&*event_name)? {
            mlua::Value::Table(t) => t,
            _ => {
                let new_table = lua.create_table()?;
                listeners.set(event_name.as_str(), new_table.clone())?;
                new_table
            }
        };

        list.push(func)?;

        Ok(())
    })?;

    events_table.set("on", on_fn)?;
    lua.globals().set("events", events_table)?;

    Ok(())
}
