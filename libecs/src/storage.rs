use crate::CHUNK_SIZE;

type Chunk<C> = Box<[C; CHUNK_SIZE]>;

pub struct ChunkPtr {
    pub ptr: *mut (),
}


pub struct ComponentStorage<C> {
    chunks: Vec<Chunk<C>>,
}

impl<C> ComponentStorage<C> {
    pub fn new() -> Self {
        ComponentStorage { chunks: Vec::new() }
    }
}

pub trait Storage {
    fn alloc_chunk(&mut self) -> ChunkPtr;
}

impl<C> Storage for ComponentStorage<C> {
    fn alloc_chunk(&mut self) -> ChunkPtr {
        self.chunks.push(Box::new(unsafe { ::std::mem::zeroed() }));
        ChunkPtr {
            ptr: self.chunks.last_mut().unwrap().as_mut_ptr() as *mut _,
        }
    }
}
