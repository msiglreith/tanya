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
        assert_eq!(entities.len(), 1);
        // TODO:
        let group_id = self.define_group::<G>();
        let chunk_index = self.groups.alloc_slots(group_id, 1).start;
        let chunk_id = chunk_index as usize / CHUNK_SIZE;
        let chunk_base = chunk_index as usize % CHUNK_SIZE;
        let chunk = self.groups.get_component_chunks(group_id, chunk_id);
        G::build_entities(chunk, &stream, chunk_base, 0, 1);

        entities[0] = self.entities.create_entity(group_id, chunk_index);
    }

    pub fn query<Q: query::Query>(&mut self) {}
}
