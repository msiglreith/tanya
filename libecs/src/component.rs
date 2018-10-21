
use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::ops::Range;

use crate::{EntityId, GroupId, CHUNK_SIZE};
use crate::storage::{ChunkPtr, ComponentStorage, Storage};

pub type ComponentId = usize;

pub trait Component: Clone + Sized + 'static {}

pub type GroupComponentChunks = Vec<ChunkPtr>;
type Components = Vec<Box<Storage>>;
type ComponentMap = HashMap<TypeId, ComponentId>;

struct ComponentGroupData {
    chunks: Vec<GroupComponentChunks>,
    num_entities: EntityId,
}

impl ComponentGroupData {
    pub fn alloc_slots(&mut self, num: usize) -> Range<EntityId> {
        // TODO: limit checking
        let start = self.num_entities;
        self.num_entities += num as EntityId;
        let end = self.num_entities;

        start..end
    }

    pub fn get_chunk(&mut self, chunk: usize) -> &mut GroupComponentChunks {
        &mut self.chunks[chunk]
    }
}

pub struct ComponentGroups {
    groups: Vec<ComponentGroupData>,
    group_lut: HashMap<TypeId, usize>,
    components: Components,
    component_lut: ComponentMap,
}

impl ComponentGroups {
    pub fn new() -> Self {
        ComponentGroups {
            groups: Vec::new(),
            group_lut: HashMap::new(),
            components: Vec::new(),
            component_lut: HashMap::new(),
        }
    }

    pub fn define_component<C: Component>(components: &mut Components, map: &mut ComponentMap) -> ComponentId {
        let component_ty_id = TypeId::of::<C>();
        *map.entry(component_ty_id).or_insert({
            let id = components.len();
            let storage = ComponentStorage::<C>::new();
            components.push(Box::new(storage));
            id
        })
    }

    pub fn define_group<'a, G: IComponentGroup<'a>>(&mut self) -> GroupId {
        let group_ty_id = TypeId::of::<G>();
        *self.group_lut.entry(group_ty_id).or_insert({
            let id = self.groups.len();
            G::define_components(&mut self.components, &mut self.component_lut);
            self.groups.push(ComponentGroupData {
                chunks: Vec::new(),
                num_entities: 0,
            });
            id
        })
    }

    pub fn alloc_slots(&mut self, group: GroupId, num: usize) -> Range<EntityId> {
        self.groups[group].alloc_slots(num)
    }

    pub fn get_component_chunks(&mut self, group: GroupId, chunk: usize) -> &mut GroupComponentChunks {
        self.groups[group].get_chunk(chunk)
    }
}

pub trait IComponentGroup<'a>: 'static {
    type BuildStream<'a>;
    fn build_entities(
        chunks: &mut GroupComponentChunks,
        stream: &Self::BuildStream,
        chunk_base: usize,
        entity_base: usize,
        num: usize,
    );
    fn define_components(components: &mut Components, map: &mut ComponentMap);
}

impl<'a, C> IComponentGroup<'a> for C
where
    C: Component,
{
    type BuildStream = &'a [C];
    fn build_entities(
        chunks: &mut GroupComponentChunks,
        stream: &Self::BuildStream,
        chunk_base: usize,
        entity_base: usize,
        num: usize,
    ) {
        let chunk_end = chunk_base + num;
        let entity_end = entity_base + num;

        let chunk = unsafe { ::std::slice::from_raw_parts_mut(chunks[0].ptr as *mut C, CHUNK_SIZE) };
        chunk[chunk_base..chunk_end].clone_from_slice(&stream[entity_base..entity_end]);
    }

    fn define_components(components: &mut Components, map: &mut ComponentMap) {
        ComponentGroups::define_component::<C>(components, map);
    }
}

impl<'a, A, B> IComponentGroup<'a> for (A, B)
where
    A: Component,
    B: Component,
{
    type BuildStream = (&'a [A], &'a [B]);
    fn build_entities(
        chunks: &mut GroupComponentChunks,
        stream: &Self::BuildStream,
        chunk_base: usize,
        entity_base: usize,
        num: usize,
    ) {
        let chunk_end = chunk_base + num;
        let entity_end = entity_base + num;

        let chunk_a = unsafe { ::std::slice::from_raw_parts_mut(chunks[0].ptr as *mut A, CHUNK_SIZE) };
        let chunk_b = unsafe { ::std::slice::from_raw_parts_mut(chunks[1].ptr as *mut B, CHUNK_SIZE) };

        chunk_a[chunk_base..chunk_end].clone_from_slice(&(stream.0)[entity_base..entity_end]);
        chunk_b[chunk_base..chunk_end].clone_from_slice(&(stream.1)[entity_base..entity_end]);
    }

    fn define_components(components: &mut Components, map: &mut ComponentMap) {
        ComponentGroups::define_component::<A>(components, map);
        ComponentGroups::define_component::<B>(components, map);
    }
}

