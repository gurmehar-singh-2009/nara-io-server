use mlua::{Function, Lua};

pub struct WeaponDef {
    pub damage: f32,
    pub reload: f32,
    pub speed: f32,
    pub on_shoot: Function,
}

pub fn load_weapon(lua: &Lua, path: &str) -> mlua::Result<WeaponDef> {
    let table: mlua::Table = lua.load(&std::fs::read_to_string(path)?).eval()?;
    Ok(WeaponDef {
        damage: table.get("damage")?,
        reload: table.get("reload")?,
        speed: table.get("speed")?,
        on_shoot: table.get("onShoot")?,
    })
}
