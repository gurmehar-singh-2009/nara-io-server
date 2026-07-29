use crate::render::buffers::EntityInstance;

pub mod triangle;
pub mod square;
pub mod pentagon;
pub mod hexagon;
pub mod octagon;
pub mod drone;
pub mod bullet;
pub mod tank;

pub trait Entity {
    fn get_render_instance(&self) -> EntityInstance;
}
