//! Casus is a simple library containing a handful of useful generic async primitives. At present, it contains `Event` and `Waiter` primitives.
//!
//! ## Event
//!
//! The Event primitive allows a future to await the completion of an event. Once the event is completed, all futures trying to await it will immediately wake up and any future calls will immediately return until the event is reset.
//!
//! ```rs
//! use casus::Event;
//!
//! let event = Event::new();
//!
//! // this will block until Event::set is called elsewhere
//! event.wait().await;
//! ```
//!
//! ## Waiter
//!
//! The Waiter primitive simply waits to be woken up with it's return value.
//!
//! ```rs
//! use casus::Waiter;
//!
//! let waiter = Waiter::new();
//!
//! // this will block until Event::wake is called elsewhere
//! waiter.await;
//! ```

use std::{
    future::Future,
    sync::{Arc, Mutex, RwLock},
    task::{Poll, Waker},
};
/// The Event primitive allows a future to await the completion of an event. Once the event is completed, all futures trying to await it will immediately wake up and any future calls will immediately return until the event is reset.
///
/// # Example
///
/// ```rs
/// use casus::Event;
///
/// let event = Event::new();
///
/// // this will block until Event::set is called elsewhere
/// event.wait().await;
/// ```

#[derive(Debug)]
pub struct Event {
    state: RwLock<bool>,
    waiters: Mutex<Vec<Waiter<()>>>,
}

impl Event {
    /// Creates a new `Event`
    ///
    /// # Example
    /// ```rs
    /// use casus::Event;
    ///
    /// let event = Event::new();
    /// ```
    pub fn new() -> Self {
        Self {
            state: RwLock::new(false),
            waiters: Mutex::new(vec![]),
        }
    }

    /// Waits for an event to be set
    ///
    /// # Example
    /// ```rs
    /// // will return when `Event::set` is called
    /// event.wait().await;
    /// ```
    pub async fn wait(&self) -> bool {
        let state = *self.state.read().unwrap();
        if !state {
            let fut = Waiter::new();
            {
                let mut waiters = self.waiters.lock().unwrap();
                waiters.push(fut.clone());
            }
            fut.await;
        }
        true
    }

    /// Sets the event and returns all current and future waiters until the event is reset
    ///
    /// # Example
    /// ```rs
    /// event.set();
    /// ```
    pub fn set(&self) {
        {
            let mut state = self.state.write().unwrap();
            *state = true;
        }
        for i in self.waiters.lock().unwrap().iter() {
            i.wake(());
        }
    }

    /// Clears the event, allowing waiters to start waiting again until the event is set
    ///
    /// # Example
    /// ```rs
    /// event.clear();
    /// ```
    pub fn clear(&self) {
        *self.state.write().unwrap() = false;
    }

    /// Checks if the event is set
    ///
    /// # Example
    /// ```rs
    /// if !event.is_set() {
    ///     event.wait().await;
    /// }
    pub fn is_set(&self) -> bool {
        *self.state.read().unwrap()
    }
}

impl Default for Event {
    fn default() -> Self {
        Self::new()
    }
}
/// The Waiter primitive simply waits to be woken up with it's return value.
///
/// # Example
///
/// ```rs
/// use casus::Waiter;
///
/// let waiter = Waiter::new();
///
/// // this will block until Event::wake is called elsewhere
/// waiter.await;
/// ```

#[derive(Clone, Debug)]
pub struct Waiter<T>(
    #[allow(clippy::type_complexity)] Arc<Mutex<(bool, Option<Waker>, Option<T>)>>,
);

impl<T> Waiter<T> {
    /// Creates a new `Waiter`
    ///
    /// # Example
    /// ```rs
    /// use casus::Waiter;
    ///
    /// let waiter = Waiter::new();
    /// ```
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new((false, None, None))))
    }

    /// Wakes up the waiter with `T` as the return value, meaning anything awaiting the waiter will return the value T
    ///
    /// # Example
    /// ```
    /// waiter.wake(T)
    /// ```
    pub fn wake(&self, v: T) {
        let mut state = self.0.lock().unwrap();
        state.0 = true;
        state.2 = Some(v);
        if let Some(waker) = state.1.take() {
            waker.wake();
        }
    }
}

impl<T> Default for Waiter<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Future for Waiter<T> {
    type Output = T;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut state = self.0.lock().unwrap();
        if state.0 {
            Poll::Ready(state.2.take().unwrap())
        } else {
            state.1 = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
