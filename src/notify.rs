use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use futures::future::Future;
use futures::task::{self, Poll, Waker};
use std::marker::Unpin;
use std::mem::PinMut;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;

#[derive(Debug)]
struct Lock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

pub(crate) struct TryLock<'a, T: 'a> {
    __ptr: &'a Lock<T>,
}

unsafe impl<T: Send> Send for Lock<T> {}
unsafe impl<T: Send> Sync for Lock<T> {}

impl<T> Lock<T> {
    pub(crate) fn new(t: T) -> Lock<T> {
        Lock {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(t),
        }
    }

    pub(crate) fn try_lock(&self) -> Option<TryLock<T>> {
        if !self.locked.swap(true, SeqCst) {
            Some(TryLock { __ptr: self })
        } else {
            None
        }
    }
}

impl<'a, T> Deref for TryLock<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.__ptr.data.get() }
    }
}

impl<'a, T> DerefMut for TryLock<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.__ptr.data.get() }
    }
}

impl<'a, T> Drop for TryLock<'a, T> {
    fn drop(&mut self) {
        self.__ptr.locked.store(false, SeqCst);
    }
}

#[must_use = "futures do nothing unless polled"]
#[derive(Debug)]
pub struct Receiver {
    inner: Arc<Inner>,
    id: Option<usize>,
}

#[derive(Debug)]
pub struct Sender {
    inner: Arc<Inner>,
}

impl Clone for Receiver {
    fn clone(&self) -> Self {
        let id = loop {
            if self.inner.complete.load(SeqCst) {
                break None;
            }

            if let Some(mut slots) = self.inner.rx_tasks.try_lock() {
                let id = slots.len();
                slots.push(None);
                break Some(id);
            }
        };

        Receiver {
            inner: self.inner.clone(),
            id,
        }
    }
}

impl Unpin for Receiver {}
impl Unpin for Sender {}

/// Internal state of the `Receiver`/`Sender` pair above. This is all used as
/// the internal synchronization between the two for send/recv operations.
#[derive(Debug)]
struct Inner {
    complete: AtomicBool,
    rx_tasks: Lock<Vec<Option<Waker>>>,
}

pub fn channel() -> (Sender, Receiver) {
    let inner = Arc::new(Inner::new());
    let receiver = Receiver {
        inner: inner.clone(),
        id: Some(0),
    };
    let sender = Sender { inner };
    (sender, receiver)
}

impl Inner {
    fn new() -> Inner {
        Inner {
            complete: AtomicBool::new(false),
            rx_tasks: Lock::new(vec![None]),
        }
    }

    fn drop_tx(&self) {
        self.complete.store(true, SeqCst);
        if let Some(mut slots) = self.rx_tasks.try_lock() {
            for slot in &mut *slots {
                if let Some(task) = slot.take() {
                    drop(slot);
                    task.wake();
                }
            }
        }
    }

    fn recv(&self, cx: &mut task::Context, id: usize) -> Poll<()> {
        let done = if self.complete.load(SeqCst) {
            true
        } else {
            let task = cx.waker().clone();
            match self.rx_tasks.try_lock() {
                Some(mut slot) => {
                    slot[id] = Some(task);
                    false
                }
                None => true,
            }
        };

        if done || self.complete.load(SeqCst) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }

    fn drop_rx(&self, id: usize) {
        if let Some(mut slots) = self.rx_tasks.try_lock() {
            let task = slots[id].take();
            drop(slots);
            drop(task);
        }
    }
}

impl Sender {
    pub fn notify(self) {
        // dropping
    }
}

impl Drop for Sender {
    fn drop(&mut self) {
        self.inner.drop_tx()
    }
}

impl Future for Receiver {
    type Output = ();

    fn poll(self: PinMut<Self>, cx: &mut task::Context) -> Poll<()> {
        match self.id {
            Some(id) => self.inner.recv(cx, id),
            None => Poll::Ready(()),
        }
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.inner.drop_rx(id);
        }
    }
}
