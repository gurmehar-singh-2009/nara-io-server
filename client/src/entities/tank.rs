use glam::Vec2;

use crate::{entities::Entity, render::buffers::EntityInstance};

pub struct Tank {
    pub id: u32,
    pub pos: Vec2,
    pub render_pos: Vec2,
    pub rot: f32,
}

impl Entity for Tank {
    fn get_render_instance(&self) -> crate::render::buffers::EntityInstance {
        // tank body
        EntityInstance {
            position: self.pos.to_array(),
            size: [100., 100.],
            rotation: self.rot,
            shape_type: 0,
            sides: 4,
            fill_color: [1., 1., 1., 1.],
            border_color: [1., 1., 1., 1.],
            border_thickness: 4.,
            extra_param: 1.,
        }

        // barrels: TODO
        // they get a little more compliated since i have to adjust
        // based on what tank they have
    }
}
