#![feature(generic_associated_types)]
#![feature(vec_resize_with)]
#![feature(dbg_macro)]

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
pub type ChunkId = usize;
pub type SlotId = usize;

const GENERATION_INVALID: Generation = Generation::max_value();
const ENTITY_COMP_ID: ComponentId = 0;

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
    chunk: ChunkId,
    slot: SlotId,
}

struct EntityComponent {
    id: EntityId,
}

pub trait Component: Clone + Sized + 'static {}

trait Storage {
    fn resize(&mut self, num_chunks: usize);
    fn alloc_chunks(&mut self, dst: &mut Vec<ChunkPtr>, chunks: Range<usize>);
    fn shift(&self, chunk: &ChunkPtr, slots: Range<usize>, amount: usize);
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

    fn shift(&self, chunk_raw: &ChunkPtr, slots: Range<usize>, amount: usize) {
        assert!(slots.start >= amount);
        assert!(slots.end <= CHUNK_SIZE);

        let chunk =
            unsafe { ::std::slice::from_raw_parts_mut::<C>(chunk_raw.ptr as *mut _, CHUNK_SIZE) };

        unsafe {
            ::std::ptr::copy(
                &chunk[slots.start] as *const _,
                &mut chunk[slots.start - amount] as *mut _,
                slots.end - slots.start,
            );
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
        chunk: ChunkId,
        slot_base: SlotId,
        stream: &Self::BuildStream,
        stream_base: usize,
        num: usize,
    );
}

#[derive(Debug)]
struct ChunkData {
    pub len: usize,
}

#[derive(Debug)]
pub struct GroupStorage {
    comp_chunks: HashMap<ComponentId, Vec<ChunkPtr>>,
    chunk_data: Vec<ChunkData>,
    free_chunks: Vec<usize>,
}

impl GroupStorage {
    pub fn num_chunks(&self) -> usize {
        self.chunk_data.len()
    }
    pub fn capacity(&self) -> u32 {
        (self.num_chunks() * CHUNK_SIZE) as u32
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
        let mut world = World {
            entities: Vec::new(),
            entities_free: FreeList::new(),
            group_storages: HashMap::new(),
            group_map: HashMap::new(),
            comp_storages: HashMap::new(),
            comp_map: HashMap::new(),
        };

        let entity_type_id = TypeId::of::<EntityComponent>();
        let id = ENTITY_COMP_ID;
        world
            .comp_storages
            .insert(id, Box::new(ComponentStorage::<EntityComponent>::new()));
        world.comp_map.insert(entity_type_id, id);

        world
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
                chunk_data: Vec::new(),
                free_chunks: Vec::new(),
            };
            for comp in components {
                storage.comp_chunks.insert(comp, Vec::new());
            }
            self.group_storages.insert(id, storage);
            id
        })
    }

    pub fn free_entities(&mut self, entities: &[Entity]) {
        for entity in entities {
            let entity_id = entity.id;
            self.entities_free.deallocate(entity_id..entity_id + 1);
            let entity_data = self.entities[entity_id as usize];
            assert_eq!(entity.generation, entity_data.generation);

            self.entities[entity_id as usize].generation += 1;

            let group_id = entity_data.group;
            let group = if let Some(group) = self.group_storages.get_mut(&group_id) {
                group
            } else {
                continue;
            };

            let used_chunk_slots = group.chunk_data[entity_data.chunk].len;
            debug_assert!(used_chunk_slots > 0);
            group.chunk_data[entity_data.chunk].len -= 1;

            let src_slot = used_chunk_slots - 1;
            let reposition = src_slot != entity_data.slot;
            if reposition {
                let src_entity_id = {
                    let entity_comp = &group.comp_chunks[&ENTITY_COMP_ID];
                    let entity_chunk_raw = &entity_comp[entity_data.chunk];
                    let entity_chunk = unsafe {
                        ::std::slice::from_raw_parts(
                            entity_chunk_raw.ptr as *mut EntityComponent,
                            CHUNK_SIZE,
                        )
                    };

                    dbg!(entity_chunk[src_slot].id)
                };

                println!("{:#?}", &self.entities);
                self.entities[src_entity_id as usize].slot = entity_data.slot;
                for (comp_id, chunks) in &group.comp_chunks {
                    let chunk = &chunks[entity_data.chunk];
                    self.comp_storages[&comp_id].shift(
                        chunk,
                        src_slot..src_slot + 1,
                        src_slot - entity_data.slot,
                    );
                }
            }

            if used_chunk_slots == 1 {
                // free chunk
                // TODO
            } else {
                //
            }
        }
    }

    pub fn create_entities<'a, G: IComponentGroup<'a>>(
        &mut self,
        entities: &mut [Entity],
        stream: G::BuildStream,
    ) {
        let group_id = self.define_group::<G>();
        let mut num_entities = entities.len();
        let mut cur_entity = 0;
        while num_entities > 0 {
            let entity_slots = if let Some(slots) = self.entities_free.allocate(num_entities as _) {
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
                        chunk: 0,
                        slot: 0,
                    },
                );
                continue;
            };
            let num_slots = entity_slots.end - entity_slots.start;
            let (chunk, chunk_slots) = self.alloc_group_slots::<G>(group_id, num_slots as _);

            {
                let entity_comp = unsafe {
                    ::std::slice::from_raw_parts_mut(
                        self.group_storages
                            .get_mut(&group_id)
                            .unwrap()
                            .comp_chunks
                            .get_mut(&ENTITY_COMP_ID)
                            .unwrap()[chunk]
                            .ptr as *mut EntityComponent,
                        CHUNK_SIZE,
                    )
                };

                for slot in 0..num_slots {
                    let entity = cur_entity + slot;

                    let entity_data = &mut self.entities[(entity_slots.start + slot) as usize];
                    let entity_id = (entity_slots.start + slot) as EntityId;
                    let slot_id = chunk_slots.start + slot as usize;

                    entity_comp[slot_id] = EntityComponent { id: entity_id };
                    entity_data.group = group_id;
                    entity_data.chunk = chunk;
                    entity_data.slot = slot_id;
                    entities[entity as usize] = Entity {
                        id: entity_id,
                        generation: entity_data.generation,
                    };
                }
            }

            G::fill_slots(
                &self.comp_map,
                &mut self.group_storages.get_mut(&group_id).unwrap().comp_chunks,
                chunk,
                chunk_slots.start as _,
                &stream,
                cur_entity as _,
                num_slots as _,
            );

            cur_entity += num_slots;
        }
    }

    fn alloc_group_slots<'a, G: IComponentGroup<'a>>(
        &mut self,
        group_id: GroupId,
        num: u32,
    ) -> (ChunkId, Range<SlotId>) {
        let group = self.group_storages.get_mut(&group_id).unwrap();
        match group.free_chunks.pop() {
            Some(chunk) => {
                let start_slot = group.chunk_data[chunk].len;
                let num_free = CHUNK_SIZE - start_slot;
                let num_alloc = num_free.min(num as _);
                group.chunk_data[chunk].len += num_alloc;
                if num_alloc < num_free {
                    group.free_chunks.push(chunk);
                }

                (chunk, start_slot..start_slot + num_alloc)
            }
            None => {
                let required_chunks = (num + CHUNK_SIZE as u32 - 1) / CHUNK_SIZE as u32;
                let cur_chunks = group.num_chunks();
                let components = G::define_components(&self.comp_map);
                for component in components {
                    let comp_storage = self.comp_storages.get_mut(&component).unwrap();
                    comp_storage.resize(required_chunks as _);
                    let num_chunks = group.num_chunks();
                    comp_storage.alloc_chunks(
                        group.comp_chunks.get_mut(&component).unwrap(),
                        num_chunks as usize..required_chunks as usize,
                    );
                }

                for _ in cur_chunks..cur_chunks + required_chunks as usize {
                    group.chunk_data.push(ChunkData { len: 0 });
                }

                let num_allocated = num.min(CHUNK_SIZE as _);
                if num < CHUNK_SIZE as u32 {
                    group.free_chunks.push(cur_chunks);
                }
                group.chunk_data[cur_chunks].len = num_allocated as _;

                for chunk in ((cur_chunks + 1)..(cur_chunks + required_chunks as usize)).rev() {
                    group.free_chunks.push(chunk);
                }

                (cur_chunks, 0..num_allocated as usize)
            }
        }
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
    chunk_data: &'a [ChunkData],
    cur_chunk: usize,
    cur_slot: usize,
    _marker: PhantomData<(A, B)>,
}

impl<'a, A, B> Iterator for GroupIterator2<'a, A, B>
where
    A: Component,
    B: Component,
{
    type Item = (&'a A, &'a B);

    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(self.cur_chunk < self.chunk_data.len());
        if self.cur_slot >= self.chunk_data[self.cur_chunk].len {
            self.cur_slot = 0;
            self.cur_chunk += 1;
        }

        if self.cur_chunk >= self.chunks_a.len() {
            return None;
        }

        let chunk_a = unsafe {
            ::std::slice::from_raw_parts(self.chunks_a[self.cur_chunk].ptr as *const A, CHUNK_SIZE)
        };
        let chunk_b = unsafe {
            ::std::slice::from_raw_parts(self.chunks_b[self.cur_chunk].ptr as *const B, CHUNK_SIZE)
        };
        let item = (&chunk_a[self.cur_slot], &chunk_b[self.cur_slot]);
        self.cur_slot += 1;

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
            chunk_data: &group.chunk_data,
            cur_chunk: 0,
            cur_slot: 0,
            _marker: PhantomData,
        }
    }

    fn define_components(comp_map: &HashMap<TypeId, ComponentId>) -> Vec<ComponentId> {
        let ty_a = TypeId::of::<A>();
        let ty_b = TypeId::of::<B>();

        vec![ENTITY_COMP_ID, comp_map[&ty_a], comp_map[&ty_b]]
    }

    fn fill_slots(
        comp_map: &HashMap<TypeId, ComponentId>,
        comp_chunks: &mut HashMap<ComponentId, Vec<ChunkPtr>>,
        chunk_id: ChunkId,
        slot_base: SlotId,
        stream: &Self::BuildStream,
        stream_base: usize,
        num: usize,
    ) {
        let ty_a = TypeId::of::<A>();
        let ty_b = TypeId::of::<B>();

        let comp_id_a = comp_map[&ty_a];
        let comp_id_b = comp_map[&ty_b];

        let start_slot = slot_base;
        let end_slot = start_slot + num;

        let start_entity = stream_base;
        let end_entity = start_entity + num;

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
        let mut entities2 = [Entity::INVALID; 8];
        let foo_data = (0..8).map(|a| Foo { a }).collect::<Vec<_>>();
        let foo_data2 = [Foo { a: 10 }; 8];
        let bar_data = [Bar {}; 8];
        world.create_entities::<(Foo, Bar)>(&mut entities, (&foo_data, &bar_data));

        println!("{:#?}", entities);
        println!("{:#?}", world.entities);

        for (a, b) in world.query_group::<(Foo, Bar)>() {
            println!("{:?}", (a, b));
        }

        world.free_entities(&entities[..4]);

        println!("{:#?}", world.entities);

        world.create_entities::<(Foo, Bar)>(&mut entities2, (&foo_data2, &bar_data));

        println!("{:#?}", entities);
        println!("{:#?}", entities2);
        println!("{:#?}", world.entities);

        for (a, b) in world.query_group::<(Foo, Bar)>() {
            println!("{:?}", (a, b));
        }
    }
}
