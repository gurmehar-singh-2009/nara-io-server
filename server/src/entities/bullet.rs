use glam::Vec2;

#[derive(Debug, Clone)]
struct Bullets {
    /// (current, max)
    lifetimes: Vec<Vec2>,
    damages: Vec<u32>,
    positions: Vec<Vec2>,
    velocities: Vec<Vec2>,
}

type Bullet<'a> = (&'a Vec2, &'a u32);
