use glam::Vec2;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Debug, Clone)]
pub struct Entities {
    generations: Vec<u32>,
    alive: Vec<bool>,

    positions: Vec<Vec2>,
    velocities: Vec<Vec2>,
    health: Vec<u32>,
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
        }
    }

    pub fn iter<'a>(&'a self) -> impl ParallelIterator<Item = EntityRef<'a>> {
        #[rustfmt::skip]
        (
            &self.generations,
            &self.alive,
            &self.positions,
            &self.velocities,
            &self.health,
        )
            .into_par_iter().map(|(generation, alive, position, velocity, health)| {
                EntityRef { generation, alive, position, velocity, health }
            })
    }

    pub fn iter_mut<'a>(&'a mut self) -> impl ParallelIterator<Item = EntityMut<'a>> {
        #[rustfmt::skip]
        (
            &mut self.generations,
            &mut self.alive,
            &mut self.positions,
            &mut self.velocities,
            &mut self.health,
        )
            .into_par_iter().map(|(generation, alive, position, velocity, health)| {
                EntityMut { generation, alive, position, velocity, health }
            })
    }

    pub fn iter_alive<'a>(&'a self) -> impl ParallelIterator<Item = EntityRef<'a>> {
        #[rustfmt::skip]
        (
            &self.generations,
            &self.alive,
            &self.positions,
            &self.velocities,
            &self.health,
        )
            .into_par_iter()
            .filter(move |(_, alive, _, _, _)| **alive)
            .map(|(generation, alive, position, velocity, health)| {
                EntityRef { generation, alive, position, velocity, health }
            })
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
            .filter(move |(_, alive, _, _, _)| **alive)
            .map(|(generation, alive, position, velocity, health)| {
                EntityMut { generation, alive, position, velocity, health }
            })
    }

    pub fn spawn(&mut self, position: Vec2, velocity: Vec2, health: u32) -> EntityId {
        if let Some(index) = self.alive.iter().position(|&alive| !alive) {
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
}

pub struct EntityRef<'a> {
    pub generation: &'a u32,
    pub alive: &'a bool,
    pub position: &'a Vec2,
    pub velocity: &'a Vec2,
    pub health: &'a u32,
}

pub struct EntityMut<'a> {
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
