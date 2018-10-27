#![feature(generic_associated_types)]
#![feature(vec_resize_with)]

mod free_list;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem;
use std::ops::Range;

use self::free_list::Allocator as FreeList;

const CHUNK_SIZE: usize = 128;
type Chunk<C> = Box<[C; CHUNK_SIZE]>;

#[derive(Debug)]
pub struct ChunkPtr {
    pub ptr: *mut (),
}

pub type EntityId = u32;
pub type Generation = u32;
pub type GroupId = usize;
pub type ComponentId = usize;

const GENERATION_INVALID: Generation = Generation::max_value();

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
struct EntityData {
    generation: Generation,
    group: GroupId,
    slot: u32,
}

pub trait Component: Clone + Sized + 'static {}

trait Storage {
    fn resize(&mut self, num_chunks: usize);
    fn alloc_chunks(&mut self, dst: &mut Vec<ChunkPtr>, chunks: Range<usize>);
}

pub struct ComponentStorage<C> {
    chunks: Vec<Chunk<C>>,
}

impl<C> ComponentStorage<C> {
    pub fn new() -> Self {
        ComponentStorage { chunks: Vec::new() }
    }
}

impl<C> Storage for ComponentStorage<C> {
    fn resize(&mut self, num_chunks: usize) {
        self.chunks
            .resize_with(num_chunks, || Chunk::new(unsafe { mem::uninitialized() }));
    }

    fn alloc_chunks(&mut self, dst: &mut Vec<ChunkPtr>, chunks: Range<usize>) {
        for chunk in chunks {
            dst.push(ChunkPtr {
                ptr: self.chunks[chunk].as_mut_ptr() as *mut _,
            })
        }
    }
}

pub trait IComponentGroup<'a>: 'static {
    type BuildStream<'a>;
    type Iterator<'a>;

    fn iter(comp_map: &HashMap<TypeId, ComponentId>, group: &'a GroupStorage) -> Self::Iterator;
    fn define_components(comp_map: &HashMap<TypeId, ComponentId>) -> Vec<ComponentId>;
    fn fill_slots(
        comp_map: &HashMap<TypeId, ComponentId>,
        comp_chunks: &mut HashMap<ComponentId, Vec<ChunkPtr>>,
        slot_base: usize,
        stream: &Self::BuildStream,
        stream_base: usize,
        num: usize,
    );
}

#[derive(Debug)]
pub struct GroupStorage {
    comp_chunks: HashMap<ComponentId, Vec<ChunkPtr>>,
    num_chunks: u32,
    num_slots: u32,
}

impl GroupStorage {
    pub fn capacity(&self) -> u32 {
        self.num_chunks * CHUNK_SIZE as u32
    }
}

pub struct World {
    entities: Vec<EntityData>,
    entities_free: FreeList,
    group_storages: HashMap<GroupId, GroupStorage>,
    group_map: HashMap<TypeId, GroupId>,
    comp_storages: HashMap<ComponentId, Box<Storage>>,
    comp_map: HashMap<TypeId, ComponentId>,
}

impl World {
    pub fn new() -> Self {
        World {
            entities: Vec::new(),
            entities_free: FreeList::new(),
            group_storages: HashMap::new(),
            group_map: HashMap::new(),
            comp_storages: HashMap::new(),
            comp_map: HashMap::new(),
        }
    }

    pub fn define_component<C: Component>(&mut self) -> ComponentId {
        let type_id = TypeId::of::<C>();
        *self.comp_map.entry(type_id).or_insert({
            let id = self.comp_storages.len();
            let storage = ComponentStorage::<C>::new();
            self.comp_storages.insert(id, Box::new(storage));
            id
        })
    }

    pub fn define_group<'a, G: IComponentGroup<'a>>(&mut self) -> GroupId {
        let type_id = TypeId::of::<G>();
        *self.group_map.entry(type_id).or_insert({
            let id = self.group_storages.len();
            let components = G::define_components(&self.comp_map);
            let mut storage = GroupStorage {
                comp_chunks: HashMap::new(),
                num_chunks: 0,
                num_slots: 0,
            };
            for comp in components {
                storage.comp_chunks.insert(comp, Vec::new());
            }
            self.group_storages.insert(id, storage);
            id
        })
    }

    pub fn create_entities<'a, G: IComponentGroup<'a>>(
        &mut self,
        entities: &mut [Entity],
        stream: G::BuildStream,
    ) {
        let group_id = self.define_group::<G>();
        let mut num_entities = entities.len();
        let chunk_slots = self.alloc_group_slots::<G>(group_id, num_entities as _);

        let mut cur_entity = 0;
        let mut base_chunk_slot = chunk_slots.start;
        while num_entities > 0 {
            let slots = if let Some(slots) = self.entities_free.allocate(num_entities as _) {
                let num_allocated = slots.end - slots.start;
                assert_ne!(num_allocated, 0);

                num_entities -= num_allocated as usize;
                slots
            } else {
                self.entities_free.append(num_entities as _);
                self.entities.resize(
                    self.entities.len() + num_entities,
                    EntityData {
                        generation: 0,
                        group: 0,
                        slot: 0,
                    },
                );
                continue;
            };

            let num_slots = slots.end - slots.start;
            for slot in 0..num_slots {
                let entity = cur_entity + slot;
                let entity_data = &mut self.entities[(slots.start + slot) as usize];
                entity_data.group = group_id;
                entity_data.slot = base_chunk_slot + slot as u32;
                entities[entity as usize] = Entity {
                    id: (slots.start + slot) as _,
                    generation: entity_data.generation,
                };
            }

            G::fill_slots(
                &self.comp_map,
                &mut self.group_storages.get_mut(&group_id).unwrap().comp_chunks,
                base_chunk_slot as _,
                &stream,
                cur_entity as _,
                num_slots as _,
            );

            cur_entity += num_slots;
            base_chunk_slot += num_slots as u32;
        }
    }

    fn alloc_group_slots<'a, G: IComponentGroup<'a>>(
        &mut self,
        group_id: GroupId,
        num: u32,
    ) -> Range<u32> {
        let group = self.group_storages.get_mut(&group_id).unwrap();
        let slots = group.num_slots..group.num_slots + num;
        let capacity = group.capacity();
        let required_slots = group.num_slots + num;
        if capacity < required_slots {
            let required_chunks = (required_slots + CHUNK_SIZE as u32 - 1) / CHUNK_SIZE as u32;
            let components = G::define_components(&self.comp_map);
            for component in components {
                let comp_storage = self.comp_storages.get_mut(&component).unwrap();
                comp_storage.resize(required_chunks as _);
                comp_storage.alloc_chunks(
                    group.comp_chunks.get_mut(&component).unwrap(),
                    group.num_chunks as usize..required_chunks as usize,
                );
            }
            group.num_chunks = required_chunks;
        }

        group.num_slots += num;
        slots
    }

    pub fn query_group<'a, G: IComponentGroup<'a>>(&'a self) -> G::Iterator {
        let group_ty = TypeId::of::<G>();
        let group_id = self.group_map[&group_ty];
        G::iter(&self.comp_map, self.group_storages.get(&group_id).unwrap())
    }
}

pub struct GroupIterator2<'a, A, B> {
    chunks_a: &'a [ChunkPtr],
    chunks_b: &'a [ChunkPtr],
    cur: usize,
    end: usize,
    _marker: PhantomData<(A, B)>,
}

impl<'a, A, B> Iterator for GroupIterator2<'a, A, B>
where
    A: Component,
    B: Component,
{
    type Item = (&'a A, &'a B);

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur >= self.end {
            return None;
        }

        let chunk = self.cur / CHUNK_SIZE;
        let chunk_slot = self.cur % CHUNK_SIZE;
        let chunk_a = unsafe {
            ::std::slice::from_raw_parts(self.chunks_a[chunk].ptr as *const A, CHUNK_SIZE)
        };
        let chunk_b = unsafe {
            ::std::slice::from_raw_parts(self.chunks_b[chunk].ptr as *const B, CHUNK_SIZE)
        };
        let item = (&chunk_a[chunk_slot], &chunk_b[chunk_slot]);
        self.cur += 1;

        Some(item)
    }
}

impl<'a, A, B> IComponentGroup<'a> for (A, B)
where
    A: Component,
    B: Component,
{
    type BuildStream = (&'a [A], &'a [B]);
    type Iterator = GroupIterator2<'a, A, B>;

    fn iter(comp_map: &HashMap<TypeId, ComponentId>, group: &'a GroupStorage) -> Self::Iterator {
        let ty_a = TypeId::of::<A>();
        let ty_b = TypeId::of::<B>();

        let comp_id_a = comp_map[&ty_a];
        let comp_id_b = comp_map[&ty_b];

        let chunks_a = group.comp_chunks.get(&comp_id_a).unwrap();
        let chunks_b = group.comp_chunks.get(&comp_id_b).unwrap();

        GroupIterator2 {
            chunks_a: &chunks_a,
            chunks_b: &chunks_b,
            cur: 0,
            end: group.num_slots as _,
            _marker: PhantomData,
        }
    }

    fn define_components(comp_map: &HashMap<TypeId, ComponentId>) -> Vec<ComponentId> {
        let ty_a = TypeId::of::<A>();
        let ty_b = TypeId::of::<B>();

        vec![comp_map[&ty_a], comp_map[&ty_b]]
    }

    fn fill_slots(
        comp_map: &HashMap<TypeId, ComponentId>,
        comp_chunks: &mut HashMap<ComponentId, Vec<ChunkPtr>>,
        slot_base: usize,
        stream: &Self::BuildStream,
        stream_base: usize,
        num: usize,
    ) {
        let ty_a = TypeId::of::<A>();
        let ty_b = TypeId::of::<B>();

        let comp_id_a = comp_map[&ty_a];
        let comp_id_b = comp_map[&ty_b];

        let slot_last = slot_base + num;

        let start_chunk = slot_base / CHUNK_SIZE;
        let end_chunk = (slot_last + CHUNK_SIZE - 1) / CHUNK_SIZE;

        let mut start_slot = slot_base;
        for chunk_id in start_chunk..end_chunk {
            let end_slot = ((chunk_id + 1) * CHUNK_SIZE).min(slot_last);
            let start_entity = start_slot - slot_base + stream_base;
            let end_entity = start_entity + (end_slot - start_slot);

            {
                let comp_a = comp_chunks.get_mut(&comp_id_a).unwrap(); // TODO: slow
                let chunk = unsafe {
                    ::std::slice::from_raw_parts_mut(comp_a[chunk_id].ptr as *mut A, CHUNK_SIZE)
                };

                chunk[start_slot..end_slot].clone_from_slice(&(stream.0)[start_entity..end_entity]);
            }

            {
                let comp_b = comp_chunks.get_mut(&comp_id_b).unwrap(); // TODO: slow
                let chunk = unsafe {
                    ::std::slice::from_raw_parts_mut(comp_b[chunk_id].ptr as *mut B, CHUNK_SIZE)
                };

                chunk[start_slot..end_slot].clone_from_slice(&(stream.1)[start_entity..end_entity]);
            }

            start_slot = end_slot;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, Debug)]
    struct Foo {
        a: usize,
    }
    impl Component for Foo {}

    #[derive(Copy, Clone, Debug)]
    struct Bar {}
    impl Component for Bar {}

    #[test]
    fn allocate_entities_simple() {
        let mut world = World::new();
        world.define_component::<Foo>();
        world.define_component::<Bar>();
        let mut entities = [Entity::INVALID; 8];
        let foo_data = [Foo { a: 4 }; 8];
        let bar_data = [Bar {}; 8];
        world.create_entities::<(Foo, Bar)>(&mut entities, (&foo_data, &bar_data));

        println!("{:?}", entities);
        println!("{:?}", world.entities);
        println!("{:?}", world.group_storages);

        for (a, b) in world.query_group::<(Foo, Bar)>() {
            println!("{:?}", (a, b));
        }
    }
}
