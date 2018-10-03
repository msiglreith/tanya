use crate::notify;
use futures::future::FutureObj;
use futures::task::Spawn;
use futures::task::SpawnError;
use std::future::Future;
use std::sync::{Arc, Mutex};

pub use rayon::ThreadPoolBuilder;

pub struct PoolInner(pub rayon::ThreadPool);
pub type Pool = Arc<Mutex<PoolInner>>;

pub struct JobSystem {
    pool: Pool,
}

impl JobSystem {
    pub fn new(pool: rayon::ThreadPool) -> Self {
        JobSystem {
            pool: Arc::new(Mutex::new(PoolInner(pool))),
        }
    }

    pub fn scope<OP, R>(&mut self, op: OP) -> R
    where
        OP: FnOnce(Scope) -> R + Send,
        R: Send,
    {
        let registry = unsafe { self.pool.lock().unwrap().0.registry() };
        let tasks = Scope {
            pool: self.pool.clone(),
        };

        registry.in_worker(|_, _| op(tasks))
    }
}

#[derive(Clone)]
pub struct Scope {
    pub pool: Pool,
}

impl Spawn for Scope {
    fn spawn_obj(&mut self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        self.pool.lock().unwrap().0.spawn_obj(future)
    }
}

pub struct Job {
    pub(crate) recv: notify::Receiver,
}
