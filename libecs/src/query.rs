use std::marker::PhantomData;
use crate::component::IComponentGroup;

pub struct Not<C>(PhantomData<C>);

pub trait Query {}

impl<'a, G> Query for G where G: IComponentGroup<'a> {}

