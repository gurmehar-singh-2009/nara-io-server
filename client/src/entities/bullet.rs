use crate::{
    entities::Entity,
    render::{buffers::EntityInstance, colours::DARK_THEME},
};

pub struct Bullet {
    pub id: u32,
    pub pos: glam::Vec2,
    pub last_pos: glam::Vec2,
    pub render_pos: glam::Vec2,
    pub rot: f32,
    pub last_rot: f32,
    pub render_rot: f32,
    pub last_update_time: f64,
}

impl Bullet {
    pub fn new(id: u32, x: f32, y: f32) -> Self {
        let pos = glam::Vec2::new(x, y);
        Self {
            id,
            pos,
            last_pos: pos,
            render_pos: pos,
            rot: 0.0,
            last_rot: 0.0,
            render_rot: 0.0,
            last_update_time: 0.0,
        }
    }
}

impl Entity for Bullet {
    fn get_render_instances(&self) -> Vec<EntityInstance> {
        vec![EntityInstance {
            position: [self.render_pos.x, self.render_pos.y],
            size: [20., 20.],
            rotation: self.render_rot,
            shape_type: 0,
            sides: 0,
            fill_color: DARK_THEME.bullet,
            border_color: DARK_THEME.tank_outline,
            border_thickness: 2.,
            extra_param: 1.,
        }]
    }
}
