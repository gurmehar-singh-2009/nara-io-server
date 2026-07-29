use std::{cell::RefCell, collections::HashMap, rc::Rc};

use mlua::{Function, Lua};

use crate::{
    entities::entity::Entities,
    game::scheduler::Scheduler,
    scripting::registery::{WeaponRegistry, register_commands},
};

const PRELUDE: &str = "function wait(seconds) return coroutine.yield(seconds) end";

pub struct Scripting {
    pub lua: Lua,
    pub weapons: WeaponRegistry,
    pub tanks: Option<()>,
    pub abilities: Option<()>,
    pub commands: Rc<RefCell<HashMap<String, Function>>>,
    pub scheduler: Scheduler,
}

impl Scripting {
    pub fn new(entities: Entities) -> mlua::Result<Self> {
        let lua = Lua::new();
        lua.set_app_data(entities);
        lua.load(PRELUDE).exec();

        let commands = register_commands(&lua)?;

        let weapons = WeaponRegistry::load_all(&lua, "content/weapons")?;
        // let tanks = TankRegistry::load_all(&lua, "content/tanks")?;
        // let abilities = AbilityRegistry::load_all(&lua, "content/abilities")?;
        lua.load(&std::fs::read_to_string("content/commands.lua")?)
            .exec()?;

        Ok(Self {
            lua,
            weapons,
            tanks: None,
            abilities: None,
            commands,
            scheduler: Scheduler::default(),
        })
    }

    pub fn entities_mut(&self) -> mlua::AppDataRefMut<Entities> {
        self.lua
            .app_data_mut::<Entities>()
            .expect("entities app data missing")
    }
}
