use crate::{
    entities::Entity,
    render::{buffers::EntityInstance, colours::DARK_THEME},
};

pub struct Pentagon {
    pub id: u32,
    pub pos: glam::Vec2,
    pub last_pos: glam::Vec2,
    pub render_pos: glam::Vec2,
    pub rot: f32,
    pub last_update_time: f64,
}

impl Pentagon {
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

impl Entity for Pentagon {
    fn get_render_instance(&self) -> EntityInstance {
        EntityInstance {
            position: [self.render_pos.x, self.render_pos.y],
            size: [60., 60.],
            rotation: self.rot,
            shape_type: 3,
            sides: 5,
            fill_color: DARK_THEME.pentagon,
            border_color: DARK_THEME.border,
            border_thickness: 4.,
            extra_param: 1.,
        }
    }
}
