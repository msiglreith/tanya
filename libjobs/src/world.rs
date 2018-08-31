use crate::resource::{Resource, ResourceTy};
use std::cell::UnsafeCell;
use std::collections::HashMap;

pub(crate) struct ResourceData(pub(crate) UnsafeCell<Box<Resource>>);

pub struct World {
    pub(crate) resources: HashMap<ResourceTy, ResourceData>,
}
unsafe impl Sync for World {}

impl World {
    pub fn new() -> Self {
        World {
            resources: HashMap::new(),
        }
    }

    pub fn add_resource<R: Resource>(&mut self, r: R) {
        let key = ResourceTy::new::<R>();
        self.resources
            .insert(key, ResourceData(UnsafeCell::new(Box::new(r))));
    }
}
