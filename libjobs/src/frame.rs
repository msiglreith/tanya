use crate::jobs::{self, Job, Scope};
use crate::resource::{self, Resource, ResourceTy};
use crate::world::{World};
use crate::notify;
use futures::future::FutureExt;
use std::mem;
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::collections::hash_map::Entry::{Vacant, Occupied};
use futures::prelude::SpawnExt;

#[derive(Copy, Clone, Debug)]
pub enum Access {
    Shared,
    Exclusive,
}

#[derive(Debug)]
enum AccessPattern {
    Read(Vec<JobId>),
    Write(JobId),
    RaW {
        write: JobId,
        reads: Vec<JobId>, // non-empty!
    },
}

impl AccessPattern {
    fn collect_jobs(&self, access: Access, recvs: &[Job]) -> Vec<notify::Receiver> {
        match (self, access) {
            (AccessPattern::Read(jobs), Access::Exclusive) => jobs.iter().map(|j| recvs[*j].recv.clone()).collect::<Vec<_>>(),
            (AccessPattern::Read(_), Access::Shared) => vec![],
            (AccessPattern::Write(job), _) => vec![recvs[*job].recv.clone()],
            (AccessPattern::RaW { write, .. }, Access::Shared) => vec![recvs[*write].recv.clone()],
            (AccessPattern::RaW { reads, .. }, Access::Exclusive) => reads.iter().map(|j| recvs[*j].recv.clone()).collect::<Vec<_>>(),
        }
    }
}

type JobId = usize;
pub type WorldId = usize;
pub type ResourceId = (WorldId, ResourceTy);
pub type Frame<'a, T> = futures::future::FutureObj<'a, T>;

#[derive(Debug)]
pub struct AccessMap {
    map: HashMap<ResourceId, Access>,
}

impl AccessMap {
    pub fn new() -> Self {
        AccessMap {
            map: HashMap::new(),
        }
    }

    fn add(&mut self, resource: ResourceId, access: Access) {
        match self.map.entry(resource) {
            Occupied(_) => panic!("Resource ({:?}) already accessed. Attempt to access ({:?}) failed!", resource, access),
            Vacant(entry) => { entry.insert(access); }
        }
    }
}

struct State {
    worlds: Vec<* mut World>,
    access: AccessMap,
    jobs: Vec<Job>,
}

pub struct FrameBuilder {
    state: RefCell<State>,
    pool: jobs::Pool,
    access_history: HashMap<ResourceId, AccessPattern>,
}

impl FrameBuilder {
    pub fn new(scope: &Scope) -> Self {
        FrameBuilder {
            state: RefCell::new(State {
                worlds: Vec::new(),
                access: AccessMap::new(),
                jobs: Vec::new(),
            }),
            pool: scope.pool.clone(),
            access_history: HashMap::new(),
        }
    }

    pub fn query<R: Resource>(&self, world_id: usize) -> ResourceHandle<R> {
        let key = ResourceTy::new::<R>();
        ResourceHandle {
            id: (world_id, key),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn spawn_job<F>(&mut self, f: F) -> notify::Receiver
    where
        F: Future<Output = ()> + 'static + Send,
    {
        let state = &mut self.state.borrow_mut();
        let access = mem::replace(&mut state.access, AccessMap::new());
        let access_history = &mut self.access_history;
        let jobs = &mut state.jobs;
        let (sender, recv) = notify::channel();
        let job = Job {
            recv: recv.clone(),
        };
        jobs.push(job);

        let job_id = jobs.len() - 1;

        // Find data dependencies with previous jobs in the same frame.
        let wait = {
            let recvs = access.map
                .iter()
                .filter_map(|(id, access)| {
                    match access_history.entry(*id) {
                        Occupied(mut entry) => {
                            let slots = entry.get().collect_jobs(*access, &jobs);

                            // Update
                            match access {
                                Access::Shared => {
                                    let e = entry.get_mut();
                                    if let AccessPattern::Read(ref mut jobs) = e {
                                        jobs.push(job_id);
                                    } else if let AccessPattern::RaW { reads, .. } = e {
                                        reads.push(job_id);
                                    } else if let AccessPattern::Write(write) = e {
                                        *e = AccessPattern::RaW { write: *write, reads: vec![job_id] }
                                    }
                                },
                                Access::Exclusive => {
                                    entry.insert(AccessPattern::Write(job_id));
                                },
                            };

                            Some(slots)
                        },
                        Vacant(entry) => {
                            let pattern = match access {
                                Access::Shared => AccessPattern::Read(vec![job_id]),
                                Access::Exclusive => AccessPattern::Write(job_id),
                            };

                            entry.insert(pattern);
                            None
                        },
                    }
                })
                .flatten()
                .collect::<Vec<_>>();

            async move {
                for recv in recvs {
                    await!(recv);
                }
            }
        };

        let signal = async {
            sender.notify();
        };
        {
            let mut pool = self.pool.lock().unwrap();
            SpawnExt::spawn(&mut pool.0, wait.then(|_| f.then(move |_| signal))).unwrap();
        }

        recv
    }

    pub fn access(&self, world: &mut World) -> WorldHandle {
        let worlds = &mut self.state.borrow_mut().worlds;
        let id = worlds.len();
        worlds.push(world);

        WorldHandle {
            world: id,
        }
    }

    pub fn dispatch(self) -> Frame<'static, ()> {
        let jobs = &mut self.state.borrow_mut().jobs;

        let mut f = Frame::new(Box::new(futures::future::ready(())));

        while let Some(Job { recv }) = jobs.pop() {
            let result = f.join(async { await!(recv); });
            f = Frame::new(Box::new(result.map(|_| ())));
        }

        f
    }

    fn access_resource(&self, id: ResourceId, access: Access) {
        self.state.borrow_mut().access.add(id, access);
    }
}

pub struct WorldHandle {
    pub world: usize,
}

impl WorldHandle {
    /// Query a resource from the world.
    pub fn query<R: Resource>(&self) -> ResourceHandle<R> {
        let key = ResourceTy::new::<R>();
        ResourceHandle {
            id: (self.world, key),
            _marker: std::marker::PhantomData,
        }
    }
}

pub struct ResourceHandle<R> {
    id: ResourceId,
    _marker: std::marker::PhantomData<R>,
}

impl<R> ResourceHandle<R> {
    pub fn read(&self, builder: &FrameBuilder) -> resource::Read<R> {
        let (world, key) = self.id;
        let resource = unsafe { &(*builder.state.borrow().worlds[world]) }.resources[&key].0.get();
        builder.access_resource(self.id, Access::Shared);
        resource::Read::new(resource as *const _)
    }

    pub fn read_write(&self, builder: &FrameBuilder) -> resource::ReadWrite<R> {
        let (world, key) = self.id;
        let resource = unsafe { &(*builder.state.borrow().worlds[world]) }.resources[&key].0.get();
        builder.access_resource(self.id, Access::Exclusive);
        resource::ReadWrite::new(resource)
    }

    pub fn id(&self) -> ResourceId {
        self.id
    }
}