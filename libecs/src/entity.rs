use crate::{EntityId, Generation, GroupId};

const GENERATION_INVALID: Generation = Generation::max_value();

pub struct Entity {
    id: EntityId,
    generation: Generation,
}

impl Entity {
    pub const INVALID: Entity = Entity {
        id: 0,
        generation: GENERATION_INVALID,
    };
}

struct EntityData {
    generation: Generation,
    group: GroupId,
    chunk_index: u32,
}

pub(crate) struct EntityList {
    entities: Vec<EntityData>,
    next_free: Option<u32>,
}

impl EntityList {
    pub fn new() -> Self {
        EntityList {
            entities: Vec::new(),
            next_free: None,
        }
    }

    pub fn create_entity(&mut self, group: GroupId, chunk_index: u32) -> Entity {
        match self.next_free.take() {
            Some(id) => {
                let num_entities = self.entities.len() as u32;
                let entity = &mut self.entities[id as usize];
                let generation = entity.generation;
                let next = entity.chunk_index;

                if next < num_entities {
                    self.next_free = Some(next);
                }

                entity.group = group;
                entity.chunk_index = chunk_index;

                Entity {
                    id: id as _,
                    generation,
                }
            }
            None => {
                let id = self.entities.len() as _;
                debug_assert!(id < EntityId::max_value());

                let generation = 0;

                self.entities.push(EntityData {
                    generation,
                    group,
                    chunk_index,
                });

                Entity { id, generation }
            }
        }
    }
}
