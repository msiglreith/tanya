#![feature(
    arbitrary_self_types,
    async_await,
    await_macro,
    futures_api,
    fnbox,
    pin
)]

pub mod frame;
pub mod jobs;
pub mod notify;
pub mod prelude;
pub mod resource;
pub mod world;

#[macro_export]
macro_rules! spawn_job {
    ($frame : expr, || $body:expr) => {
        spawn_job!($frame, | | $body)
    };

    ($frame : expr, |$k0:ident $e0:ident| $body:expr) => {
        spawn_job!($frame, |$k0 $e0,| $body)
    };

    ($frame : expr, |$k0:ident $e0:ident $(,$k:ident $e:ident)*| $body:expr) => {
        spawn_job!($frame, |$k0 $e0, $($k $e,)*|)
    };

    ($frame : expr, |$($keyword:ident $e:ident,)*| $body:expr) => {
        {
            expand_args!($frame, $($keyword $e,)*);
            ($frame).spawn_job(async move { $body })
        }
    };
}

#[macro_export]
macro_rules! expand_args {
    ($frame:expr,) => { };
    ($frame:expr, ref $arg:ident, $($rest:tt)*) => {
        let $arg = $arg.read(&$frame);
        expand_args!($frame, $($rest)*)
    };
    ($frame:expr, mut $arg:ident, $($rest:tt)*) => {
        let mut $arg = $arg.read_write(&$frame);
        expand_args!($frame, $($rest)*)
    };
}
