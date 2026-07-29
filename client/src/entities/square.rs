use crate::{entities::Entity, render::buffers::EntityInstance};

struct Square {
    pub x: f32,
    pub y: f32,
    pub rot: f32,
}

impl Entity for Square {
    fn get_render_instance(&self) -> crate::render::buffers::EntityInstance {
        EntityInstance {
            position: [self.x, self.y],
            size: [40., 40.],
            rotation: self.rot,
            shape_type: 1,
            sides: 4,
            fill_color: [1., 0., 0., 1.],
            border_color: [0., 1., 0., 1.],
            border_thickness: 4.,
            extra_param: 1.,
        }
    }
}