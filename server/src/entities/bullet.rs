use glam::Vec2;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::entities::entity::EntityId;

#[derive(Debug, Clone)]
pub struct Bullets {
    positions: Vec<Vec2>,
    velocities: Vec<Vec2>,
    lifetimes: Vec<f32>,
    damages: Vec<u32>,
    owners: Vec<EntityId>,
    net_ids: Vec<u32>,
    next_net_id: u32,
}

impl Bullets {
    pub fn new() -> Self {
        Self {
            positions: vec![],
            velocities: vec![],
            lifetimes: vec![],
            damages: vec![],
            owners: vec![],
            net_ids: vec![],
            next_net_id: 0x40000000,
        }
    }

    pub fn spawn(
        &mut self,
        position: Vec2,
        velocity: Vec2,
        damage: u32,
        lifetime: f32,
        owner: EntityId,
    ) -> u32 {
        let id = self.next_net_id;
        self.next_net_id += 1;

        self.positions.push(position);
        self.velocities.push(velocity);
        self.lifetimes.push(lifetime);
        self.damages.push(damage);
        self.owners.push(owner);
        self.net_ids.push(id);

        id
    }

    pub fn len(&self) -> usize {
        self.positions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    pub fn remove(&mut self, i: usize) {
        self.positions.swap_remove(i);
        self.velocities.swap_remove(i);
        self.lifetimes.swap_remove(i);
        self.damages.swap_remove(i);
        self.owners.swap_remove(i);
        self.net_ids.swap_remove(i);
    }

    pub fn tick(&mut self, dt: f32) {
        (
            &mut self.positions,
            &mut self.velocities,
            &mut self.lifetimes,
        )
            .into_par_iter()
            .for_each(|(pos, vel, life)| {
                *pos += *vel * dt;
                *life -= dt;
            });

        let mut i = 0;
        while i < self.lifetimes.len() {
            if self.lifetimes[i] <= 0.0 {
                self.remove(i);
            } else {
                i += 1;
            }
        }
    }

    pub fn iter_indexed(&self) -> impl Iterator<Item = (usize, &Vec2, &u32, &EntityId)> {
        self.positions
            .iter()
            .zip(self.damages.iter())
            .zip(self.owners.iter())
            .enumerate()
            .map(|(i, ((pos, dmg), owner))| (i, pos, dmg, owner))
    }

    pub fn iter(&self) -> impl Iterator<Item = (u32, &Vec2)> {
        self.positions
            .iter()
            .zip(self.net_ids.iter())
            .map(|(pos, id)| (*id, pos))
    }
}
