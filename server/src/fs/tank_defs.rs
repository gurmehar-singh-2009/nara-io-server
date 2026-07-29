use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Tank {
    id: u32,
    name: String,
    upgrade_message: String,
    level_requirement: u32,
    upgrades: Vec<u32>,
    flags: TankFlags,
    invisibility: Option<serde_json::Value>,
    field_factor: f32,
    absorbtion_factor: f32,
    max_health: u32,
    pre_addon: u32,
    post_addon: u32,
    sides: u32,
    barrels: Vec<Barrel>,
    stats: Vec<Stat>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TankFlags {
    invisibility: bool,
    zoom_ability: bool,
    can_shoot: bool,
    dev_only: bool,
}

#[derive(Deserialize, Debug)]
struct Stat {
    name: String,
    max: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Barrel {
    angle: f64,
    offset: f64,
    size: f64,
    width: f64,
    delay: f64,
    reload: f64,
    recoil: f64,
    flags: BarrelFlags,
    trapezoid_direction: f32,
    addon: f32,
    bullet: Bullet,
    drone_count: Option<u32>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct BarrelFlags {
    is_trapezoid: bool,
    force_fire: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Bullet {
    #[serde(rename = "type")]
    bullet_type: String,
    health: f64,
    damage: f64,
    speed: f64,
    scatter_rate: f64,
    life_length: f64,
    absorbtion_factor: f64,
    size_ratio: f64,
}

pub fn aaa() {
    let json_str = fs::read_to_string("src/entities/tank_defs.json").unwrap();
    let tanks: Vec<Tank> = serde_json::from_str(&json_str).unwrap();

    println!("Loaded {} tanks successfully!", tanks.len());

    for tank in &tanks {
        println!("Tank {}", tank.name);
    }
}
