use crate::entities::entity::EntityId;

/// (current, needed for next level)
pub type Xp = (u32, u32);

#[derive(Debug, Clone)]
pub struct Tanks {
    ids: Vec<EntityId>,
    names: Vec<String>,
    xp: Vec<Xp>,
    aim: Vec<f32>,
    sparse: Vec<Option<usize>>,
}

pub struct TankRef<'a> {
    pub name: &'a str,
    pub xp: &'a Xp,
    pub aim: &'a f32,
}

pub struct TankMut<'a> {
    pub name: &'a mut String,
    pub xp: &'a mut Xp,
    pub aim: &'a mut f32,
}

impl Tanks {
    pub fn new(max_tanks: usize) -> Self {
        Self {
            ids: Vec::with_capacity(max_tanks),
            names: Vec::with_capacity(max_tanks),
            xp: Vec::with_capacity(max_tanks),
            aim: Vec::with_capacity(max_tanks),
            sparse: Vec::with_capacity(max_tanks),
        }
    }

    fn ensure_sparse_capacity(&mut self, index: usize) {
        if index >= self.sparse.len() {
            self.sparse.resize(index + 1, None);
        }
    }

    pub fn insert(&mut self, id: EntityId, name: String) {
        self.ensure_sparse_capacity(id.index);
        if self.sparse[id.index].is_some() {
            return;
        }

        let slot = self.ids.len();
        self.ids.push(id);
        self.names.push(name);
        self.aim.push(0.);
        self.xp.push((0, 250));
        self.sparse[id.index] = Some(slot);
    }

    pub fn remove(&mut self, id: EntityId) -> bool {
        let Some(slot) = self.sparse.get(id.index).copied().flatten() else {
            return false;
        };

        let last = self.ids.len() - 1;
        self.ids.swap(slot, last);
        self.names.swap(slot, last);
        self.xp.swap(slot, last);
        self.aim.swap(slot, last);

        self.ids.pop();
        self.names.pop();
        self.xp.pop();
        self.aim.pop();

        self.sparse[id.index] = None;

        if slot != last {
            let moved_id = self.ids[slot];
            self.sparse[moved_id.index] = Some(slot);
        }

        true
    }

    pub fn get(&self, id: EntityId) -> Option<TankRef<'_>> {
        let slot = (*self.sparse.get(id.index)?)?;
        Some(TankRef {
            name: &self.names[slot],
            xp: &self.xp[slot],
            aim: &self.aim[slot],
        })
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<TankMut<'_>> {
        let slot = (*self.sparse.get(id.index)?)?;
        Some(TankMut {
            name: &mut self.names[slot],
            xp: &mut self.xp[slot],
            aim: &mut self.aim[slot],
        })
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}
