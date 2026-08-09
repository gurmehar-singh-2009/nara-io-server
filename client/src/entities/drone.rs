use crate::{
    entities::Entity,
    render::{buffers::EntityInstance, colours::DARK_THEME},
};

pub struct Drone {
    pub id: u32,
    pub pos: glam::Vec2,
    pub last_pos: glam::Vec2,
    pub render_pos: glam::Vec2,
    pub rot: f32,
    pub last_update_time: f64,
}

impl Drone {
    pub fn new(id: u32, x: f32, y: f32) -> Self {
        let pos = glam::Vec2::new(x, y);
        Self {
            id,
            pos,
            last_pos: pos,
            render_pos: pos,
            rot: 0.0,
            last_update_time: 0.0,
        }
    }
}

impl Entity for Drone {
    fn get_render_instance(&self) -> EntityInstance {
        EntityInstance {
            position: [self.render_pos.x, self.render_pos.y],
            size: [20., 20.],
            rotation: self.rot,
            shape_type: 3,
            sides: 3,
            fill_color: DARK_THEME.team_blue,
            border_color: DARK_THEME.tank_outline,
            border_thickness: 3.,
            extra_param: 1.,
        }
    }
}
