
#![feature(arbitrary_self_types, async_await, await_macro, futures_api, pin)]


extern crate futures;
#[macro_use]
extern crate mopa;

use std::future::Future;
use futures::executor::ThreadPool;
use futures::prelude::*;
use mopa::Any;
use std::any::TypeId;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::task::Executor;
use futures::executor::spawn;
use futures::executor::spawn_with_handle;
use futures::executor::block_on;

pub trait IntoFuture {
    /// The future that this type can be converted into.
    type Future: Future<Output=Self::Output>;

    /// The item that the future may resolve with.
    type Output;

    /// Consumes this object and produces a future.
    fn into_future(self) -> Self::Future;
}

impl<F> IntoFuture for F where F: Future {
    type Future = Self;
    type Output = <Self as Future>::Output;
    fn into_future(self) -> Self { self }
}

pub trait Resource: Any + Send + Sync + 'static {}
mopafy!(Resource);

impl<T> Resource for T where T: Any + Send + Sync {}

pub struct ResourceData(pub UnsafeCell<Box<Resource>>);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResourceId(pub TypeId);

impl ResourceId {
    /// Creates a new resource id from a given type.
    pub fn new<T: Resource>() -> Self {
        ResourceId(TypeId::of::<T>())
    }
}

pub struct ResourceHandle<'a, R: 'a >(*mut Box<Resource>, std::marker::PhantomData<&'a R>);

unsafe impl<'a, R> Send for ResourceHandle<'a, R> {}

impl<'a, R: Resource> std::ops::Deref for ResourceHandle<'a, R> {
    type Target = R;
    fn deref(&self) -> &R {
        unsafe { (*self.0).downcast_ref_unchecked() }
    }
}

impl<'a, R: Resource> std::ops::DerefMut for ResourceHandle<'a, R> {
    fn deref_mut(&mut self) -> &mut R {
        unsafe { (*self.0).downcast_mut_unchecked() }
    }
}

pub struct World {
    resources: HashMap<ResourceId, ResourceData>,
}

impl World {
    pub fn new() -> Self {
        World {
            resources: HashMap::new(),
        }
    }

    pub fn add_resource<R: Resource>(&mut self, r: R) {
        let key = ResourceId::new::<R>();
        self.resources
            .insert(key, ResourceData(UnsafeCell::new(Box::new(r))));
    }
}

enum ResourceAccess {
    Shared,
    Exclusive,
}

pub struct WorldFrame<'a> {
    world: &'a mut World,
    access: HashMap<TypeId, ResourceAccess>,
    tasks: std::sync::Mutex<Vec<std::future::FutureObj<'static, ()>>>,
}

impl<'a> WorldFrame<'a> {
    pub fn spawn<F, T>(&self, fnc: F)
    where
        F: FnOnce() -> T,
        T: IntoFuture<Output=()>,
        T::Future: Send + 'a,
    {
        let task = fnc();
        self.tasks.lock().unwrap().push(std::future::FutureObj::new(Box::new(task.into_future())));
        println!("push!", );
    }

    pub fn read_mut<R: Resource>(&self) -> ResourceHandle<'a, R> {
        let key = ResourceId::new::<R>();
        ResourceHandle(self.world.resources[&key].0.get(), std::marker::PhantomData)
    }
}

pub struct JobSystem {
    pool: ThreadPool,
}

pub struct FrameFuture {}

impl JobSystem {
    pub fn enqueue<F>(&mut self, world: &mut World, job: F) -> FrameFuture
    where
        F: FnOnce(&mut WorldFrame),
    {
        let mut frame = WorldFrame {
            world,
            access: HashMap::new(),
            tasks: std::sync::Mutex::new(Vec::new()),
        };

        job(&mut frame);

        let mut tasks = frame.tasks.lock().unwrap();
        let results = tasks.drain(..).map(|t| block_on(spawn_with_handle(t))).collect::<Vec<_>>();
        for result in results {
            block_on(result);
        }

        FrameFuture {}
    }
}

async fn foo() {
    println!("hm");
}

fn main() {
    let pool = futures::executor::ThreadPoolBuilder::new().create().unwrap();
    let mut data: Vec<u32> = vec![0, 2, 1, 4];

    let mut world = World::new();
    world.add_resource(data);

    let mut jobs = JobSystem { pool };

    let game_frame0 = jobs.enqueue(&mut world, |mut frame| {
        for i in 0..10 {
            let physics_check = frame.spawn(|| {
                let rays = frame.read_mut::<Vec<u32>>();

                frame.spawn(|| {
                    async {
                        println!("{:?}", 6);
                    }
                });

                async move {
                    println!("hm");
                    println!("{:?}", &*rays);
                    for ray in &*rays {
                        let r = ray.clone();
                        block_on(spawn_with_handle(async move { println!("{:?}", (i, r)) }));
                    }
                }
            });
        }
    });
}
