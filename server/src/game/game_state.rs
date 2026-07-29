use rayon::iter::ParallelIterator;

use crate::{
    entities::entity::{Entities, EntityId},
    scripting::scripting::Scripting,
};

pub struct GameState {
    pub scripting: Scripting,
}

impl GameState {
    pub fn new() -> mlua::Result<Self> {
        Ok(Self {
            scripting: Scripting::new(Entities::new())?,
        })
    }

    pub async fn game_loop(&mut self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));

        loop {
            self.scripting
                .entities_mut()
                .iter_alive_mut()
                .for_each(|_e| { /* physics */ });
            self.scripting.scheduler.on_tick();

            interval.tick().await;
        }
    }
}
