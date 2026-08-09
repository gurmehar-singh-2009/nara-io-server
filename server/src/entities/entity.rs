use glam::Vec2;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

use crate::entities::{
    bullet::Bullets,
    shape::{ShapeKind, Shapes},
    tank::Tanks,
};

#[derive(Debug, Clone)]
pub struct Entities {
    pub generations: Vec<u32>,
    pub alive: Vec<bool>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub health: Vec<u32>,
    pub free: Vec<usize>,

    pub tanks: Tanks,
    pub bullets: Bullets,
    pub shapes: Shapes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId {
    pub index: usize,
    pub generation: u32,
}

impl Entities {
    pub fn new() -> Self {
        Self {
            generations: vec![],
            alive: vec![],
            positions: vec![],
            velocities: vec![],
            health: vec![],
            free: vec![],
            tanks: Tanks::new(256),
            bullets: Bullets::new(),
            shapes: Shapes::new(2048),
        }
    }

    pub fn spawn_tank(
        &mut self,
        position: Vec2,
        velocity: Vec2,
        health: u32,
        name: String,
    ) -> EntityId {
        let id = self.spawn(position, velocity, health);
        self.tanks.insert(id, name);
        id
    }

    pub fn spawn_shape(
        &mut self,
        center: Vec2,
        kind: ShapeKind,
        health: u32,
        rotation_speed: f32,
        xp_reward: u32,
        orbit_radius: f32,
        orbit_angle: f32,
        orbit_speed: f32,
    ) -> EntityId {
        let initial_pos = center + Vec2::new(orbit_angle.cos(), orbit_angle.sin()) * orbit_radius;
        let id = self.spawn(initial_pos, Vec2::ZERO, health);
        self.shapes.insert(
            id,
            kind,
            rotation_speed,
            xp_reward,
            center,
            orbit_radius,
            orbit_angle,
            orbit_speed,
        );
        id
    }
    
    pub fn despawn(&mut self, id: EntityId) -> bool {
        if !self.is_alive(id) {
            return false;
        }
        self.alive[id.index] = false;
        self.tanks.remove(id);
        self.shapes.remove(id);
        self.free.push(id.index);
        true
    }

    pub fn is_alive(&self, id: EntityId) -> bool {
        self.alive.get(id.index).copied().unwrap_or(false)
            && self.generations[id.index] == id.generation
    }

    pub fn iter<'a>(&'a self) -> impl ParallelIterator<Item = EntityRef<'a>> {
        #[rustfmt::skip]
        (&self.generations, &self.alive, &self.positions, &self.velocities, &self.health)
            .into_par_iter()
            .enumerate()
            .map(|(index, (generation, alive, position, velocity, health))| {
                EntityRef { index, generation, alive, position, velocity, health }
            })
    }

    pub fn iter_alive<'a>(&'a self) -> impl ParallelIterator<Item = EntityRef<'a>> {
        #[rustfmt::skip]
        (&self.generations, &self.alive, &self.positions, &self.velocities, &self.health)
            .into_par_iter()
            .enumerate()
            .filter(|(_, (_, alive, _, _, _))| **alive)
            .map(|(index, (generation, alive, position, velocity, health))| {
                EntityRef { index, generation, alive, position, velocity, health }
            })
    }

    pub fn get(&self, id: EntityId) -> Option<EntityRef<'_>> {
        if !self.is_alive(id) {
            return None;
        }
        Some(EntityRef {
            index: id.index,
            generation: &self.generations[id.index],
            alive: &self.alive[id.index],
            position: &self.positions[id.index],
            velocity: &self.velocities[id.index],
            health: &self.health[id.index],
        })
    }

    pub fn health_mut(&mut self, id: EntityId) -> Option<&mut u32> {
        if !self.is_alive(id) {
            return None;
        }
        Some(&mut self.health[id.index])
    }

    pub fn iter_alive_mut<'a>(&'a mut self) -> impl ParallelIterator<Item = EntityMut<'a>> {
        #[rustfmt::skip]
        (
            &mut self.generations,
            &mut self.alive,
            &mut self.positions,
            &mut self.velocities,
            &mut self.health,
        )
            .into_par_iter()
            .enumerate()
            .filter(|(_, (_, alive, _, _, _))| **alive)
            .map(|(index, (generation, alive, position, velocity, health))| {
                EntityMut { index, generation, alive, position, velocity, health }
            })
    }

    pub fn spawn(&mut self, position: Vec2, velocity: Vec2, health: u32) -> EntityId {
        if let Some(index) = self.free.pop() {
            self.generations[index] += 1;
            self.alive[index] = true;
            self.positions[index] = position;
            self.velocities[index] = velocity;
            self.health[index] = health;

            EntityId {
                index,
                generation: self.generations[index],
            }
        } else {
            let index = self.alive.len();
            self.generations.push(0);
            self.alive.push(true);
            self.positions.push(position);
            self.velocities.push(velocity);
            self.health.push(health);

            EntityId {
                index,
                generation: 0,
            }
        }
    }

    pub fn speed_of(&self, id: EntityId) -> f32 {
        0.
    }

    pub fn set_speed(&self, id: EntityId, vel: f32) {}

    pub fn iter_alive_mut_with_tanks<'a>(
        &'a mut self,
    ) -> (impl ParallelIterator<Item = EntityMut<'a>> + 'a, &'a Tanks) {
        let Entities {
            generations,
            alive,
            positions,
            velocities,
            health,
            tanks,
            ..
        } = self;

        let iter = (generations, alive, positions, velocities, health)
            .into_par_iter()
            .enumerate()
            .filter(|(_, (_, alive, _, _, _))| **alive)
            .map(
                |(index, (generation, alive, position, velocity, health))| EntityMut {
                    index,
                    generation,
                    alive,
                    position,
                    velocity,
                    health,
                },
            );

        (iter, &*tanks)
    }
}

pub struct EntityRef<'a> {
    pub index: usize,
    pub generation: &'a u32,
    pub alive: &'a bool,
    pub position: &'a Vec2,
    pub velocity: &'a Vec2,
    pub health: &'a u32,
}

pub struct EntityMut<'a> {
    pub index: usize,
    pub generation: &'a mut u32,
    pub alive: &'a mut bool,
    pub position: &'a mut Vec2,
    pub velocity: &'a mut Vec2,
    pub health: &'a mut u32,
}

pub fn spawn_entity(
    entities: &mut Entities,
    position: Vec2,
    velocity: Vec2,
    health: u32,
) -> EntityId {
    entities.spawn(position, velocity, health)
}
