use crate::render::buffers::EntityInstance;

pub mod bullet;
pub mod drone;
pub mod hexagon;
pub mod octagon;
pub mod pentagon;
pub mod square;
pub mod tank;
pub mod triangle;

pub trait Entity {
    fn get_render_instance(&self) -> EntityInstance;
}
