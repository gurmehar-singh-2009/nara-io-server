use glam::Vec2;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::entities::entity::EntityId;

#[derive(Debug, Clone)]
pub struct Bullets {
    positions: Vec<Vec2>,
    velocities: Vec<Vec2>,
    lifetimes: Vec<f32>,
    damages: Vec<u32>,
    owners: Vec<EntityId>, // in the future we can do something cool with multiple owners
}

impl Bullets {
    pub fn new() -> Self {
        Self {
            positions: vec![],
            velocities: vec![],
            lifetimes: vec![],
            damages: vec![],
            owners: vec![],
        }
    }

    pub fn spawn(
        &mut self,
        position: Vec2,
        velocity: Vec2,
        damage: u32,
        lifetime: f32,
        owner: EntityId,
    ) {
        self.positions.push(position);
        self.velocities.push(velocity);
        self.lifetimes.push(lifetime);
        self.damages.push(damage);
        self.owners.push(owner);
    }

    pub fn len(&self) -> usize {
        self.positions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
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
                self.positions.swap_remove(i);
                self.velocities.swap_remove(i);
                self.lifetimes.swap_remove(i);
                self.damages.swap_remove(i);
                self.owners.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Vec2, &u32, &EntityId)> {
        self.positions
            .iter()
            .zip(self.damages.iter())
            .zip(self.owners.iter())
            .map(|((pos, dmg), owner)| (pos, dmg, owner))
    }
}
