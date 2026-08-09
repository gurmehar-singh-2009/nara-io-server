use std::{collections::HashMap, time::Duration};

use glam::Vec2;
use nanorand::Rng;
use paris::error;
use rayon::iter::ParallelIterator;
use shared::packets::{
    PACKET_SEED,
    client_bound::{
        AddEntityPacket, BarrelDef, EntityType, LeaderboardPacket, PlayerStatsPacket,
        RemoveEntityPacket, TankSpec, UpdateEntityPacket, UpdateEntityPacketData,
    },
};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    entities::{
        connections::Connections,
        entity::{Entities, EntityId},
        shape::ShapeKind,
        spatial_hash::{HashEntity, SpatialHash},
    },
    scripting::scripting::Scripting,
};

#[derive(Debug)]
pub enum GameEvents {
    PlayerSpawn { id: u32, name: String },
    PlayerDisconnect { id: u32 },
    PlayerMovement { id: u32, dir: Option<f32> },
    PlayerAutoFire { id: u32, enabled: bool },
    PlayerAim { id: u32, dir: f32 },
}

const TANK_TYPE: &str = "Basic";
const TANK_SPEED_SCALE: f32 = 50.0;
const TANK_RADIUS: f32 = 20.0;
const BULLET_RADIUS: f32 = 8.0;
const BULLET_LIFETIME: f32 = 1.5;
const MAX_LEVEL: u32 = 45;

const MAP_BOUND: f32 = 2500.0;
const MAX_SHAPES: usize = 1500;

pub struct GameState {
    pub scripting: Scripting,
    pub spatial_hash: SpatialHash,
    pub game_channel_recv: UnboundedReceiver<GameEvents>,
    pub connections: Connections,
    players: HashMap<u32, EntityId>,
    rnd: nanorand::WyRand,
    tick_count: u64,
}

impl GameState {
    pub fn new(
        game_channel_recv: UnboundedReceiver<GameEvents>,
        connections: Connections,
    ) -> mlua::Result<Self> {
        Ok(Self {
            scripting: Scripting::new(Entities::new())?,
            spatial_hash: SpatialHash::new(),
            game_channel_recv,
            connections,
            players: HashMap::new(),
            rnd: nanorand::WyRand::new(),
            tick_count: 0,
        })
    }

    pub async fn game_loop(&mut self) {
        let tick_rate = Duration::from_millis(100);
        let dt = tick_rate.as_secs_f32();
        let mut interval = tokio::time::interval(tick_rate);

        loop {
            self.tick_count += 1;
            while let Ok(msg) = self.game_channel_recv.try_recv() {
                match &msg {
                    GameEvents::PlayerSpawn { id, name } => {
                        let x = (self.rnd.generate::<f32>() * 1000.0) - 500.0;
                        let y = (self.rnd.generate::<f32>() * 1000.0) - 500.0;

                        let entity_id = self.scripting.entities_mut().spawn_tank(
                            Vec2::from_array([x, y]),
                            Vec2::ZERO,
                            100,
                            name.clone(),
                        );

                        self.players.insert(*id, entity_id);

                        let barrels = vec![BarrelDef {
                            x: 0.0,
                            y: 0.0,
                            angle: 0.0,
                            width: 18.0,
                            length: 40.0,
                        }];
                        let my_packet = AddEntityPacket::new(
                            *id,
                            EntityType::Player,
                            x,
                            y,
                            1,
                            name.to_string(),
                            true,
                            barrels.clone(),
                            PACKET_SEED as u64,
                        );
                        let other_packet = AddEntityPacket::new(
                            *id,
                            EntityType::Player,
                            x,
                            y,
                            1,
                            name.to_string(),
                            false,
                            barrels.clone(),
                            PACKET_SEED as u64,
                        );

                        self.connections.send_to(*id, my_packet);
                        self.connections
                            .broadcast_with_exceptions(other_packet, &[*id]);

                        for (&other_id, &other_entity_id) in self.players.iter() {
                            if other_id == *id {
                                continue;
                            }
                            let entities = self.scripting.entities_mut();
                            let Some(entity) = entities.get(other_entity_id) else {
                                continue;
                            };
                            let Some(tank) = entities.tanks.get(other_entity_id) else {
                                continue;
                            };
                            let existing_packet = AddEntityPacket::new(
                                other_id,
                                EntityType::Player,
                                entity.position.x,
                                entity.position.y,
                                *tank.level,
                                tank.name.to_string(),
                                false,
                                tank.barrels.clone(),
                                PACKET_SEED as u64,
                            );
                            self.connections.send_to(*id, existing_packet);
                        }
                    }
                    GameEvents::PlayerDisconnect { id } => {
                        if let Some(entity_id) = self.players.remove(id) {
                            self.scripting.entities_mut().despawn(entity_id);
                        }
                        self.connections.remove(*id);
                    }
                    GameEvents::PlayerMovement { id, dir } => {
                        if let Some(&entity_id) = self.players.get(id) {
                            let mut entities = self.scripting.entities_mut();
                            if entities.is_alive(entity_id) {
                                entities.tanks.set_movement_dir(entity_id, *dir);
                            }
                        }
                    }
                    GameEvents::PlayerAutoFire { id, enabled } => {
                        if let Some(&entity_id) = self.players.get(id) {
                            let mut entities = self.scripting.entities_mut();
                            if entities.is_alive(entity_id) {
                                entities.tanks.set_auto_fire(entity_id, *enabled);
                            }
                        }
                    }
                    GameEvents::PlayerAim { id, dir } => {
                        if let Some(&entity_id) = self.players.get(id) {
                            let mut entities = self.scripting.entities_mut();
                            if entities.is_alive(entity_id) {
                                if let Some(tank) = entities.tanks.get_mut(entity_id) {
                                    *tank.aim = *dir;
                                }
                            }
                        }
                    }
                }
                if let Err(err) = self.scripting.dispatch_event(msg) {
                    error!("Lua event error: {err}");
                }
            }

            let entity_to_conn: HashMap<EntityId, u32> =
                self.players.iter().map(|(&k, &v)| (v, k)).collect();

            let tank_spec = self
                .scripting
                .tanks
                .0
                .get(TANK_TYPE)
                .map(|def| def.spec.clone());
            let base_speed = tank_spec
                .as_ref()
                .map(|s| s.speed * TANK_SPEED_SCALE)
                .unwrap_or(250.0);

            {
                let mut entities = self.scripting.entities_mut();
                let Entities {
                    shapes, positions, ..
                } = &mut *entities;
                shapes.tick(dt, positions);
            }

            {
                let mut entities = self.scripting.entities_mut();
                for i in 0..entities.alive.len() {
                    if !entities.alive[i] {
                        continue;
                    }
                    let id = EntityId {
                        index: i,
                        generation: entities.generations[i],
                    };

                    let max_health = entities
                        .tanks
                        .get(id)
                        .map(|t| *t.max_health)
                        .or_else(|| {
                            entities.shapes.get(id).map(|s| match s.kind {
                                ShapeKind::Square => 10,
                                ShapeKind::Triangle => 30,
                                ShapeKind::Pentagon => 100,
                            })
                        })
                        .unwrap_or(100);

                    if let Some(health) = entities.health_mut(id) {
                        if *health > 0 && *health < max_health {
                            let regen_amount = (max_health / 100).max(1);
                            *health = (*health + regen_amount).min(max_health);
                        }
                    }
                }
            }

            self.resolve_entity_collisions();

            {
                let mut entities = self.scripting.entities_mut();
                for center in entities.shapes.centers.iter_mut() {
                    if center.x < -MAP_BOUND {
                        center.x = -MAP_BOUND;
                    }
                    if center.x > MAP_BOUND {
                        center.x = MAP_BOUND;
                    }
                    if center.y < -MAP_BOUND {
                        center.y = -MAP_BOUND;
                    }
                    if center.y > MAP_BOUND {
                        center.y = MAP_BOUND;
                    }
                }
            }

            let entity_updates: Vec<UpdateEntityPacketData> = {
                let mut entities = self.scripting.entities_mut();
                let mut updates = Vec::new();

                for i in 0..entities.alive.len() {
                    if !entities.alive[i] {
                        continue;
                    }
                    let id = EntityId {
                        index: i,
                        generation: entities.generations[i],
                    };
                    let tank_data = entities
                        .tanks
                        .get(id)
                        .map(|t| (*t.move_dir, *t.aim, *t.level));
                    let shape_data = entities.shapes.get(id).map(|s| *s.rotation);

                    let health = entities.get(id).map(|e| *e.health).unwrap_or(0);
                    let max_health = if let Some(tank) = entities.tanks.get(id) {
                        *tank.max_health
                    } else if let Some(shape) = entities.shapes.get(id) {
                        match shape.kind {
                            ShapeKind::Square => 10,
                            ShapeKind::Triangle => 30,
                            ShapeKind::Pentagon => 100,
                        }
                    } else {
                        100
                    };

                    if let Some((move_dir, aim, level)) = tank_data {
                        let current_max_speed = base_speed * (1.0 + (level - 1) as f32 * 0.02);

                        if let Some(dir) = move_dir {
                            let target = Vec2::from_angle(dir) * current_max_speed;
                            let delta = target - entities.velocities[i];
                            let dist = delta.length();
                            let max_step = 800.0 * dt;
                            if dist > max_step {
                                entities.velocities[i] += delta / dist * max_step;
                            } else {
                                entities.velocities[i] = target;
                            }
                        } else {
                            let speed = entities.velocities[i].length();
                            let drop = 500.0 * dt;
                            if speed > drop {
                                let vel = entities.velocities[i];
                                entities.velocities[i] -= vel / speed * drop;
                            } else {
                                entities.velocities[i] = Vec2::ZERO;
                            }
                        }

                        let vel = entities.velocities[i];
                        entities.velocities[i] = vel.clamp(
                            Vec2::splat(-current_max_speed),
                            Vec2::splat(current_max_speed),
                        );
                        let vel = entities.velocities[i];
                        entities.positions[i] += vel * dt;

                        if entities.positions[i].x < -MAP_BOUND {
                            entities.positions[i].x = -MAP_BOUND;
                            entities.velocities[i].x = 0.0;
                        }
                        if entities.positions[i].x > MAP_BOUND {
                            entities.positions[i].x = MAP_BOUND;
                            entities.velocities[i].x = 0.0;
                        }
                        if entities.positions[i].y < -MAP_BOUND {
                            entities.positions[i].y = -MAP_BOUND;
                            entities.velocities[i].y = 0.0;
                        }
                        if entities.positions[i].y > MAP_BOUND {
                            entities.positions[i].y = MAP_BOUND;
                            entities.velocities[i].y = 0.0;
                        }

                        if let Some(conn_id) = entity_to_conn.get(&id).copied() {
                            let scale = 1.0 + (level - 1) as f32 * 0.08;
                            updates.push(UpdateEntityPacketData {
                                id: conn_id,
                                entity_type: EntityType::Player,
                                x: entities.positions[i].x,
                                y: entities.positions[i].y,
                                rot: aim,
                                scale,
                                health,
                                max_health,
                            });
                        }
                    } else if let Some(rot) = shape_data {
                        let net_id = 0x80000000 | (i as u32);
                        updates.push(UpdateEntityPacketData {
                            id: net_id,
                            entity_type: EntityType::Shape,
                            x: entities.positions[i].x,
                            y: entities.positions[i].y,
                            rot,
                            scale: 1.0,
                            health,
                            max_health,
                        });
                    }
                }

                for (net_id, pos) in entities.bullets.iter() {
                    updates.push(UpdateEntityPacketData {
                        id: net_id,
                        entity_type: EntityType::Bullet,
                        x: pos.x,
                        y: pos.y,
                        rot: 0.0,
                        scale: 1.0,
                        health: 1,
                        max_health: 1,
                    });
                }
                updates
            };

            self.connections
                .broadcast(UpdateEntityPacket::new(entity_updates, PACKET_SEED as u64));

            {
                let entities = self.scripting.entities_mut();
                for (&conn_id, &entity_id) in self.players.iter() {
                    if let Some(tank) = entities.tanks.get(entity_id) {
                        let health = entities.get(entity_id).map(|e| *e.health).unwrap_or(0);
                        let packet = PlayerStatsPacket::new(
                            *tank.level,
                            tank.xp.0,
                            tank.xp.1,
                            health,
                            *tank.max_health,
                            PACKET_SEED as u64,
                        );
                        self.connections.send_to(conn_id, packet);
                    }
                }
            }

            if self.tick_count % 10 == 0 {
                let entities = self.scripting.entities_mut();
                let mut leaderboard: Vec<(String, u32)> = self
                    .players
                    .iter()
                    .filter_map(|(_, &entity_id)| {
                        let tank = entities.tanks.get(entity_id)?;
                        Some((tank.name.to_string(), tank.xp.0))
                    })
                    .collect();
                leaderboard.sort_by(|a, b| b.1.cmp(&a.1));
                leaderboard.truncate(10);
                let packet = LeaderboardPacket::new(leaderboard, PACKET_SEED as u64);
                self.connections.broadcast(packet);
            }

            self.spatial_hash.clear();
            {
                let entities = self.scripting.entities_mut();
                for e in entities.iter_alive().collect::<Vec<_>>() {
                    let id = EntityId {
                        index: e.index,
                        generation: *e.generation,
                    };
                    self.spatial_hash
                        .insert(HashEntity::Entity(id), e.position.x, e.position.y);
                }
            }

            let substeps = 5;
            let sub_dt = dt / substeps as f32;
            for _ in 0..substeps {
                self.scripting.entities_mut().bullets.tick(sub_dt);
                self.resolve_bullet_collisions();
            }

            if let Some(spec) = &tank_spec {
                self.fire_auto_weapons(Some(spec), dt);
            } else {
                self.fire_auto_weapons(None, dt);
            }

            self.scripting.scheduler.on_tick();
            self.tick_shape_spawns(dt);

            interval.tick().await;
        }
    }

    fn resolve_entity_collisions(&mut self) {
        let mut entities = self.scripting.entities_mut();
        let mut collidables = Vec::new();
        for i in 0..entities.alive.len() {
            if !entities.alive[i] {
                continue;
            }
            let id = EntityId {
                index: i,
                generation: entities.generations[i],
            };
            let is_tank = entities.tanks.get(id).is_some();
            let radius = if is_tank {
                TANK_RADIUS
            } else if let Some(shape) = entities.shapes.get(id) {
                match shape.kind {
                    ShapeKind::Square => 15.0,
                    ShapeKind::Triangle => 20.0,
                    ShapeKind::Pentagon => 35.0,
                }
            } else {
                continue;
            };
            collidables.push((id, entities.positions[i], radius, is_tank));
        }

        let mut collision_hits: Vec<(EntityId, u32)> = Vec::new();

        for i in 0..collidables.len() {
            let (id1, pos1, r1, is_tank1) = &collidables[i];
            for j in (i + 1)..collidables.len() {
                let (id2, pos2, r2, is_tank2) = &collidables[j];
                let delta = *pos2 - *pos1;
                let dist = delta.length();
                let min_dist = r1 + r2;
                if dist < min_dist && dist > 0.0 {
                    let push = (min_dist - dist);
                    let dir = delta / dist;
                    if *is_tank1 && *is_tank2 {
                        entities.velocities[id1.index] -= dir * push * 5.0;
                        entities.velocities[id2.index] += dir * push * 5.0;
                        collision_hits.push((*id1, 2));
                        collision_hits.push((*id2, 2));
                    } else if *is_tank1 && !*is_tank2 {
                        entities.shapes.push_center(*id2, dir * push * 0.5);
                        let vel = entities.velocities[id1.index];
                        entities.velocities[id1.index] -= dir * vel.dot(dir) * 2.0;
                        collision_hits.push((*id1, 1));
                        collision_hits.push((*id2, 15));
                    } else if !*is_tank1 && *is_tank2 {
                        entities.shapes.push_center(*id1, -dir * push * 0.5);
                        let vel = entities.velocities[id2.index];
                        entities.velocities[id2.index] -= -dir * vel.dot(-dir) * 2.0;
                        collision_hits.push((*id1, 15));
                        collision_hits.push((*id2, 1));
                    } else {
                        entities.shapes.push_center(*id1, -dir * push * 0.25);
                        entities.shapes.push_center(*id2, dir * push * 0.25);
                    }
                }
            }
        }
        drop(entities);

        if !collision_hits.is_empty() {
            let entity_to_conn: HashMap<EntityId, u32> =
                self.players.iter().map(|(&k, &v)| (v, k)).collect();
            let mut entities = self.scripting.entities_mut();
            for (target_id, damage) in collision_hits {
                if let Some(health) = entities.health_mut(target_id) {
                    let was_alive = *health > 0;
                    *health = health.saturating_sub(damage);
                    if was_alive && *health == 0 {
                        let is_tank = entities.tanks.get(target_id).is_some();
                        let net_id = if is_tank {
                            entity_to_conn.get(&target_id).copied().unwrap_or(0)
                        } else {
                            0x80000000 | (target_id.index as u32)
                        };
                        let entity_type = if is_tank {
                            EntityType::Player
                        } else {
                            EntityType::Shape
                        };
                        let packet =
                            RemoveEntityPacket::new(net_id, entity_type, PACKET_SEED as u64);
                        self.connections.broadcast(packet);

                        entities.despawn(target_id);
                    }
                }
            }
        }
    }

    fn tick_shape_spawns(&mut self, dt: f32) {
        if self.players.is_empty() {
            return;
        }
        let mut entities = self.scripting.entities_mut();
        while entities.shapes.len() < MAX_SHAPES {
            let x = (self.rnd.generate::<f32>() * MAP_BOUND * 2.0) - MAP_BOUND;
            let y = (self.rnd.generate::<f32>() * MAP_BOUND * 2.0) - MAP_BOUND;
            if x.abs() < 200.0 && y.abs() < 200.0 {
                continue;
            }
            let rand_val = self.rnd.generate::<f32>();
            let (kind, health, rot_speed, xp) = if rand_val < 0.75 {
                (ShapeKind::Square, 10, 0.05, 10)
            } else if rand_val < 0.95 {
                (ShapeKind::Triangle, 30, 0.07, 25)
            } else {
                (ShapeKind::Pentagon, 100, 0.03, 130)
            };
            let center = Vec2::new(x, y);
            let orbit_radius = self.rnd.generate::<f32>() * 50.0 + 20.0;
            let orbit_angle = self.rnd.generate::<f32>() * std::f32::consts::TAU;
            let orbit_speed = (self.rnd.generate::<f32>() * 0.4 + 0.1)
                * if self.rnd.generate::<bool>() {
                    1.0
                } else {
                    -1.0
                };
            let entity_id = entities.spawn_shape(
                center,
                kind,
                health,
                rot_speed,
                xp,
                orbit_radius,
                orbit_angle,
                orbit_speed,
            );
            let kind_u32 = match kind {
                ShapeKind::Square => 1,
                ShapeKind::Triangle => 2,
                ShapeKind::Pentagon => 3,
            };
            let net_id = 0x80000000 | (entity_id.index as u32);
            let packet = AddEntityPacket::new(
                net_id,
                EntityType::Shape,
                center.x,
                center.y,
                kind_u32,
                String::new(),
                false,
                vec![],
                PACKET_SEED as u64,
            );
            self.connections.broadcast(packet);
        }
    }

    fn resolve_bullet_collisions(&mut self) {
        let hits: Vec<(usize, EntityId, u32, EntityId)> = {
            let entities = self.scripting.entities_mut();
            let mut hits = Vec::new();
            for (bullet_index, pos, damage, owner) in entities.bullets.iter_indexed() {
                let nearby = self.spatial_hash.get_nearby(pos.x, pos.y, 40.0);
                for candidate in nearby {
                    let HashEntity::Entity(target_id) = candidate else {
                        continue;
                    };
                    if target_id == *owner {
                        continue;
                    }
                    let Some(target) = entities.get(target_id) else {
                        continue;
                    };
                    let target_radius = if entities.tanks.get(target_id).is_some() {
                        TANK_RADIUS
                    } else if let Some(shape) = entities.shapes.get(target_id) {
                        match shape.kind {
                            ShapeKind::Square => 15.0,
                            ShapeKind::Triangle => 20.0,
                            ShapeKind::Pentagon => 35.0,
                        }
                    } else {
                        continue;
                    };
                    if pos.distance(*target.position) > target_radius + BULLET_RADIUS {
                        continue;
                    }
                    hits.push((bullet_index, target_id, *damage, *owner));
                    break;
                }
            }
            hits
        };

        if hits.is_empty() {
            return;
        }

        let mut entities = self.scripting.entities_mut();
        let entity_to_conn: HashMap<EntityId, u32> =
            self.players.iter().map(|(&k, &v)| (v, k)).collect();

        for (_, target_id, damage, owner) in &hits {
            if let Some(health) = entities.health_mut(*target_id) {
                let was_alive = *health > 0;
                *health = health.saturating_sub(*damage);

                if was_alive && *health == 0 {
                    let is_tank = entities.tanks.get(*target_id).is_some();
                    let is_shape = entities.shapes.get(*target_id).is_some();

                    if is_tank {
                        let target_tank = entities.tanks.get(*target_id);
                        if let Some(target_tank) = target_tank {
                            let target_xp = target_tank.xp.0 / 2;
                            if let Some(mut owner_tank) = entities.tanks.get_mut(*owner) {
                                owner_tank.xp.0 += target_xp;
                                let start_level = *owner_tank.level;
                                while owner_tank.xp.0 >= owner_tank.xp.1
                                    && *owner_tank.level < MAX_LEVEL
                                {
                                    owner_tank.xp.0 -= owner_tank.xp.1;
                                    owner_tank.xp.1 =
                                        (owner_tank.xp.1 as f32 * 1.12).min(100000.0) as u32;
                                    *owner_tank.level += 1;
                                    *owner_tank.max_health += 10;
                                    *owner_tank.bullet_damage += 2;
                                    *owner_tank.bullet_speed += 10.0;
                                    *owner_tank.reload_time =
                                        (*owner_tank.reload_time * 0.98).max(0.1);
                                }
                                if *owner_tank.level == MAX_LEVEL {
                                    owner_tank.xp.0 = owner_tank.xp.1 - 1;
                                }

                                let levels_gained = *owner_tank.level - start_level;
                                let new_max_health = *owner_tank.max_health;
                                if levels_gained > 0 {
                                    if let Some(h) = entities.health_mut(*owner) {
                                        *h = (*h + levels_gained * 10).min(new_max_health);
                                    }
                                }
                            }
                        }
                    } else if is_shape {
                        let shape = entities.shapes.get(*target_id);
                        if let Some(shape) = shape {
                            let xp_gained = *shape.xp_reward;
                            if let Some(mut owner_tank) = entities.tanks.get_mut(*owner) {
                                owner_tank.xp.0 += xp_gained;
                                let start_level = *owner_tank.level;
                                while owner_tank.xp.0 >= owner_tank.xp.1
                                    && *owner_tank.level < MAX_LEVEL
                                {
                                    owner_tank.xp.0 -= owner_tank.xp.1;
                                    owner_tank.xp.1 =
                                        (owner_tank.xp.1 as f32 * 1.12).min(100000.0) as u32;
                                    *owner_tank.level += 1;
                                    *owner_tank.max_health += 10;
                                    *owner_tank.bullet_damage += 2;
                                    *owner_tank.bullet_speed += 10.0;
                                    *owner_tank.reload_time =
                                        (*owner_tank.reload_time * 0.98).max(0.1);
                                }
                                if *owner_tank.level == MAX_LEVEL {
                                    owner_tank.xp.0 = owner_tank.xp.1 - 1;
                                }

                                let levels_gained = *owner_tank.level - start_level;
                                let new_max_health = *owner_tank.max_health;
                                if levels_gained > 0 {
                                    if let Some(h) = entities.health_mut(*owner) {
                                        *h = (*h + levels_gained * 10).min(new_max_health);
                                    }
                                }
                            }
                        }
                    }

                    let net_id = if is_tank {
                        entity_to_conn.get(target_id).copied().unwrap_or(0)
                    } else {
                        0x80000000 | (target_id.index as u32)
                    };
                    let entity_type = if is_tank {
                        EntityType::Player
                    } else {
                        EntityType::Shape
                    };
                    let packet = RemoveEntityPacket::new(net_id, entity_type, PACKET_SEED as u64);
                    self.connections.broadcast(packet);

                    entities.despawn(*target_id);
                }
            }
        }

        let mut spent: Vec<usize> = hits.iter().map(|(b, _, _, _)| *b).collect();
        spent.sort_unstable_by(|a, b| b.cmp(a));
        spent.dedup();
        for i in spent {
            entities.bullets.remove(i);
        }
    }

    fn fire_auto_weapons(&mut self, spec: Option<&TankSpec>, dt: f32) {
        let mut guard = self.scripting.entities_mut();
        let entities: &mut Entities = &mut guard;

        let default_barrels = vec![BarrelDef {
            x: 0.0,
            y: 0.0,
            angle: 0.0,
            width: 18.0,
            length: 40.0,
        }];

        let ids: Vec<EntityId> = self.players.values().copied().collect();
        for id in ids {
            if !entities.is_alive(id) {
                continue;
            }
            let Some(entity) = entities.get(id) else {
                continue;
            };
            let position = *entity.position;

            let Some(tank) = entities.tanks.get_mut(id) else {
                continue;
            };
            if !*tank.auto_fire {
                continue;
            }

            *tank.reload_timer -= dt;
            if *tank.reload_timer > 0.0 {
                continue;
            }

            let bullet_damage = *tank.bullet_damage;
            let bullet_speed = *tank.bullet_speed;
            let reload_time = *tank.reload_time;

            *tank.reload_timer += reload_time;
            let aim = *tank.aim;

            let barrels = if tank.barrels.is_empty() {
                spec.map(|s| s.barrels.clone())
                    .unwrap_or(default_barrels.clone())
            } else {
                tank.barrels.clone()
            };

            for barrel in &barrels {
                let barrel_angle = barrel.angle.to_radians();
                let muzzle_local =
                    Vec2::new(barrel.x, barrel.y) + Vec2::from_angle(barrel_angle) * barrel.length;
                let world_angle = aim + barrel_angle;
                let muzzle_world = position + Vec2::from_angle(aim).rotate(muzzle_local);
                let velocity = Vec2::from_angle(world_angle) * bullet_speed;

                entities
                    .bullets
                    .spawn(muzzle_world, velocity, bullet_damage, BULLET_LIFETIME, id);
            }
        }
    }
}
