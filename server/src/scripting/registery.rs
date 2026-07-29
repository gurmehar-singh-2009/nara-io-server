use std::{cell::RefCell, collections::HashMap, rc::Rc};

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

pub fn register_commands(lua: &Lua) -> mlua::Result<Rc<RefCell<HashMap<String, Function>>>> {
    let commands: Rc<RefCell<HashMap<String, Function>>> = Rc::new(RefCell::new(HashMap::new()));

    let reg = commands.clone();
    let register_fn = lua.create_function(move |_, (name, func): (String, Function)| {
        reg.borrow_mut().insert(name, func);
        Ok(())
    })?;

    let table = lua.create_table()?;

    table.set("register", register_fn)?;
    lua.globals().set("commands", table)?;

    Ok(commands)
}
