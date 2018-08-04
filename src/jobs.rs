use crate::notify;
use std::future::Future;

pub struct JobSystem {
    pub(crate) pool: rayon::ThreadPool,
}

impl JobSystem {
    pub fn new(pool: rayon::ThreadPool) -> Self {
        JobSystem { pool }
    }

    pub fn scope<OP, R>(&mut self, op: OP) -> R
    where
        OP: FnOnce(Scope) -> R + Send,
        R: Send,
    {
        let registry = unsafe { self.pool.registry() };
        let tasks = Scope { system: self };

        registry.in_worker(|_, _| op(tasks))
    }
}

pub struct Scope<'a> {
    pub(crate) system: &'a mut JobSystem,
}

impl<'a> Scope<'a> {
    pub fn block_on<F: Future>(&self, f: F) -> F::Output {
        self.system.pool.block_on(f)
    }
}

pub struct Job {
    pub(crate) recv: notify::Receiver,
}
