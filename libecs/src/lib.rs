#![feature(generic_associated_types)]

pub mod component;
pub mod entity;
pub mod query;
pub mod storage;

use self::component::{ComponentGroups, IComponentGroup};
use self::entity::EntityList;

pub use self::component::Component;
pub use self::entity::Entity;

const CHUNK_SIZE: usize = 128;

pub type EntityId = u32;
pub type Generation = u32;
pub type GroupId = usize;

pub struct Entities {
    entities: EntityList,
    groups: ComponentGroups,
}

impl Entities {
    pub fn new() -> Self {
        Entities {
            entities: EntityList::new(),
            groups: ComponentGroups::new(),
        }
    }

    pub fn define_group<'a, G: IComponentGroup<'a>>(&mut self) -> GroupId {
        self.groups.define_group::<G>()
    }

    pub fn create_entities<'a, G: IComponentGroup<'a>>(
        &mut self,
        entities: &mut [Entity],
        stream: G::BuildStream,
    ) {
        let group_id = self.define_group::<G>();

        let chunk_indices = self.groups.alloc_slots(group_id, entities.len());
        let chunk_id_start = chunk_indices.start as usize / CHUNK_SIZE;
        let chunk_id_end = (chunk_indices.end as usize + CHUNK_SIZE - 1) / CHUNK_SIZE;

        let mut chunk_base = chunk_indices.start as usize;
        let mut cur_entity = 0;

        for chunk_id in chunk_id_start..chunk_id_end {
            let id_end = (chunk_indices.end as usize).min((chunk_id_start + 1) * CHUNK_SIZE);
            let start = chunk_base % CHUNK_SIZE;
            let num = id_end - chunk_base;

            let chunk = self.groups.get_component_chunks(group_id, chunk_id);
            G::build_entities(chunk, &stream, start, cur_entity, num as _);

            for e in cur_entity..cur_entity + num {
                entities[0] = self.entities.create_entity(group_id, chunk_id as _);
            }

            cur_entity += num;
            chunk_base += num;
        }
    }

    pub fn query<Q: query::Query>(&mut self) {}
}
