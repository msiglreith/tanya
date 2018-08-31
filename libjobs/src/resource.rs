use std::any::Any;
use std::any::TypeId;

pub trait Resource: Any + Send + Sync + 'static {}
impl<T> Resource for T where T: Any + Send + Sync {}

impl Resource {
    pub unsafe fn downcast_ref_unchecked<T: Resource>(&self) -> &T {
        &*(self as *const Self as *const T)
    }

    pub unsafe fn downcast_mut_unchecked<T: Resource>(&mut self) -> &mut T {
        &mut *(self as *mut Self as *mut T)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResourceTy(pub TypeId);
impl ResourceTy {
    /// Creates a new resource id from a given type.
    pub fn new<T: Resource>() -> Self {
        ResourceTy(TypeId::of::<T>())
    }
}

pub struct Read<R>(*const Box<Resource>, std::marker::PhantomData<R>);
unsafe impl<R> Send for Read<R> {}

impl<R> Read<R> {
    pub(crate) fn new(resource: *const Box<Resource>) -> Self {
        Read(resource, std::marker::PhantomData)
    }
}

impl<R: Resource> std::ops::Deref for Read<R> {
    type Target = R;
    fn deref(&self) -> &R {
        unsafe { (*self.0).downcast_ref_unchecked() }
    }
}

pub struct ReadWrite<R>(*mut Box<Resource>, std::marker::PhantomData<R>);
unsafe impl<R> Send for ReadWrite<R> {}

impl<R> ReadWrite<R> {
    pub(crate) fn new(resource: *mut Box<Resource>) -> Self {
        ReadWrite(resource, std::marker::PhantomData)
    }
}

impl<R: Resource> std::ops::Deref for ReadWrite<R> {
    type Target = R;
    fn deref(&self) -> &R {
        unsafe { (*self.0).downcast_ref_unchecked() }
    }
}

impl<R: Resource> std::ops::DerefMut for ReadWrite<R> {
    fn deref_mut(&mut self) -> &mut R {
        unsafe { (*self.0).downcast_mut_unchecked() }
    }
}

