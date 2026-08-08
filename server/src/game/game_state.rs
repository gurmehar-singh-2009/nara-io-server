use std::{collections::HashMap, time::Duration};

use glam::Vec2;
use nanorand::Rng;
use paris::error;
use rayon::iter::ParallelIterator;
use shared::packets::{PACKET_SEED, client_bound::AddEntityPacket};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    entities::{
        connections::Connections,
        entity::{Entities, EntityId},
        spatial_hash::SpatialHash,
    },
    scripting::scripting::Scripting,
};

#[derive(Debug)]
pub enum GameEvents {
    PlayerSpawn { id: u32, name: String },
    PlayerDisconnect { id: u32 },
}

pub struct GameState {
    pub scripting: Scripting,
    pub spatial_hash: SpatialHash,
    pub game_channel_recv: UnboundedReceiver<GameEvents>,
    pub connections: Connections,
    players: HashMap<u32, EntityId>,
    rnd: nanorand::WyRand,
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
        })
    }

    pub async fn game_loop(&mut self) {
        let tick_rate = Duration::from_millis(100);
        let dt = tick_rate.as_secs_f32();
        let mut interval = tokio::time::interval(tick_rate);

        loop {
            // first we handle all the events from the game channel.
            while let Ok(msg) = self.game_channel_recv.try_recv() {
                match &msg {
                    GameEvents::PlayerSpawn { id, name } => {
                        // generate random coordinate for the player to spawn.
                        let (x, y) = (self.rnd.generate::<f32>(), self.rnd.generate::<f32>());

                        let entity_id = self.scripting.entities_mut().spawn_tank(
                            Vec2::from_array([x, y]),
                            Vec2::ZERO,
                            100,
                            name.clone(),
                        );

                        self.players.insert(*id, entity_id);

                        // broadcast the spawn msg.
                        let my_packet = AddEntityPacket::new(
                            *id,
                            shared::packets::client_bound::EntityType::Player,
                            x,
                            y,
                            0,
                            name.to_string(),
                            true,
                            PACKET_SEED as u64,
                        );
                        let other_packet = AddEntityPacket::new(
                            *id,
                            shared::packets::client_bound::EntityType::Player,
                            x,
                            y,
                            0,
                            name.to_string(),
                            false,
                            PACKET_SEED as u64,
                        );

                        self.connections.send_to(*id, my_packet);
                        self.connections
                            .broadcast_with_exceptions(other_packet, &[*id]);

                        // now, we also need to send any existing players to them too
                        for (&other_id, &other_entity_id) in self.players.iter() {
                            if other_id == *id {
                                continue; // that's the player who just joined - already handled above
                            }

                            let entities = self.scripting.entities_mut();

                            let Some(entity) = entities.get(other_entity_id) else {
                                continue; // they gone, ignore
                            };

                            let Some(tank) = entities.tanks.get(other_entity_id) else {
                                continue;
                            };

                            let existing_packet = AddEntityPacket::new(
                                other_id,
                                shared::packets::client_bound::EntityType::Player,
                                entity.position.x,
                                entity.position.y,
                                0,
                                tank.name.to_string(),
                                false,
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
                }

                if let Err(err) = self.scripting.dispatch_event(msg) {
                    error!("Lua event error: {err}");
                }
            }

            self.scripting
                .entities_mut()
                .iter_alive_mut()
                .for_each(|e| {
                    *e.position += *e.velocity * dt;
                });
            self.scripting.entities_mut().bullets.tick(dt);

            self.scripting.scheduler.on_tick();
            interval.tick().await;
        }
    }
}
