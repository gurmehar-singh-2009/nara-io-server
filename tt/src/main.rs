use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use mlua::{
    thread::ThreadStatus, Function, Lua, Thread, UserData, UserDataFields, UserDataMethods,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(usize);

#[derive(Default)]
pub struct Entities {
    speed: Vec<f32>,
    health: Vec<u32>,
}

impl Entities {
    fn spawn(&mut self) -> EntityId {
        self.speed.push(5.0);
        self.health.push(100);
        EntityId(self.speed.len() - 1)
    }

    fn speed_of(&self, id: EntityId) -> f32 {
        self.speed[id.0]
    }

    fn set_speed(&mut self, id: EntityId, v: f32) {
        self.speed[id.0] = v;
    }

    fn spawn_bullet_for(&mut self, id: EntityId) {
        println!("  -> entity {:?} spawned a bullet", id);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LuaPlayer(pub EntityId);

impl UserData for LuaPlayer {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("speed", |lua, this| {
            let entities = lua.app_data_ref::<Entities>().unwrap();
            Ok(entities.speed_of(this.0))
        });
        fields.add_field_method_set("speed", |lua, this: &mut LuaPlayer, v: f32| {
            let mut entities = lua.app_data_mut::<Entities>().unwrap();
            entities.set_speed(this.0, v);
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("spawnBullet", |lua, this, ()| {
            let mut entities = lua.app_data_mut::<Entities>().unwrap();
            entities.spawn_bullet_for(this.0);
            Ok(())
        });
    }
}

#[derive(Default)]
pub struct Scheduler {
    pending: Vec<(Thread, u64)>,
    tick: u64,
}

impl Scheduler {
    fn start(&mut self, lua: &Lua, func: Function, player: LuaPlayer) -> mlua::Result<()> {
        let thread = lua.create_thread(func)?;
        self.resume(thread, player)
    }

    fn resume(&mut self, thread: Thread, args: impl mlua::IntoLuaMulti) -> mlua::Result<()> {
        let result: Option<f64> = thread.resume(args)?;
        if thread.status() == ThreadStatus::Resumable {
            let ticks = (result.unwrap_or(0.0) * 10.0) as u64; // 100ms ticks
            println!("  -> ability yielded, resuming in {} ticks", ticks);
            self.pending.push((thread, self.tick + ticks));
        } else {
            println!("  -> ability finished");
        }
        Ok(())
    }

    fn on_tick(&mut self) {
        self.tick += 1;
        let ready: Vec<_> = self
            .pending
            .iter()
            .filter(|(_, at)| *at <= self.tick)
            .map(|(t, _)| t.clone())
            .collect();
        self.pending.retain(|(_, at)| *at > self.tick);
        for thread in ready {
            let _ = self.resume(thread, ());
        }
    }
}

fn register_commands(lua: &Lua) -> mlua::Result<Arc<Mutex<HashMap<String, Function>>>> {
    let commands: Arc<Mutex<HashMap<String, Function>>> = Arc::new(Mutex::new(HashMap::new()));
    let reg = commands.clone();
    let register_fn = lua.create_function(move |_, (name, func): (String, Function)| {
        reg.lock().unwrap().insert(name, func);
        // reg.borrow_mut().insert(name, func);
        Ok(())
    })?;
    let table = lua.create_table()?;
    table.set("register", register_fn)?;
    lua.globals().set("commands", table)?;
    Ok(commands)
}

const PRELUDE: &str = "function wait(seconds) return coroutine.yield(seconds) end";

fn main() -> mlua::Result<()> {
    let lua = Lua::new();

    let mut entities = Entities::default();
    let player_id = entities.spawn();
    lua.set_app_data(entities);

    lua.load(PRELUDE).exec()?;
    let commands = register_commands(&lua)?;

    let weapon_src = r#"
        return {
            damage = 10,
            reload = 0.25,
            onShoot = function(player)
                print("Lua: onShoot called, current speed = " .. player.speed)
                player:spawnBullet()
            end,
        }
    "#;
    let weapon_table: mlua::Table = lua.load(weapon_src).eval()?;
    let on_shoot: Function = weapon_table.get("onShoot")?;
    let damage: f32 = weapon_table.get("damage")?;
    println!("loaded weapon, damage = {damage}");

    println!("--- firing weapon ---");
    on_shoot.call::<()>(LuaPlayer(player_id))?;

    let ability_src = r#"
        return {
            activate = function(player)
                print("Lua: activating, speed = " .. player.speed)
                player.speed = player.speed * 2
                wait(0.3)
                print("Lua: resumed after wait, speed = " .. player.speed)
                player.speed = player.speed / 2
                print("Lua: ability finished, speed = " .. player.speed)
            end,
        }
    "#;
    let ability_table: mlua::Table = lua.load(ability_src).eval()?;
    let activate: Function = ability_table.get("activate")?;

    println!("--- activating ability ---");
    let mut scheduler = Scheduler::default();
    scheduler.start(&lua, activate, LuaPlayer(player_id))?;

    println!("--- ticking scheduler ---");
    for i in 1..=5 {
        scheduler.on_tick();
        println!("(tick {i})");
    }

    let commands_src = r#"
        commands.register("heal", function(player)
            print("Lua: healing player, was speed " .. player.speed)
        end)
    "#;
    lua.load(commands_src).exec()?;

    println!("--- dispatching /heal ---");
    let heal_fn = commands.lock().unwrap().get("heal").unwrap().clone();
    heal_fn.call::<()>(LuaPlayer(player_id))?;

    println!("all good");
    Ok(())
}
