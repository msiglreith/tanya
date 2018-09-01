use crate::notify;
use std::future::Future;
use std::sync::{Arc, Mutex};

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

impl Scope {
    pub fn block_on<F: Future>(&self, f: F) -> F::Output {
        self.pool.lock().unwrap().0.block_on(f)
    }
}

pub struct Job {
    pub(crate) recv: notify::Receiver,
}
