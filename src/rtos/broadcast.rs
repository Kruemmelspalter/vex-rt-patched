use core::ops::{Deref, DerefMut};

use alloc::sync::{Arc, Weak};
use owner_monad::{Owner, OwnerMut};

use super::{handle_event, Event, EventHandle, GenericSleep, Mutex, Selectable};
use crate::error::Error;

/// Represents a source of data which notifies listeners on a new value.
pub struct Broadcast<T: Clone>(Arc<Mutex<BroadcastData<T>>>);

impl<T: Clone> Broadcast<T> {
    #[inline]
    /// Creates a new broadcast event with the associated initial value. Panics
    /// on failure; see [`Broadcast::try_new()`].
    pub fn new(data: T) -> Self {
        Self::try_new(data).unwrap_or_else(|err| panic!("failed to create broadcast: {:?}", err))
    }

    /// Creates a new broadcast event with the associated initial value.
    pub fn try_new(data: T) -> Result<Self, Error> {
        Ok(Self(Arc::new(Mutex::try_new(BroadcastData {
            data: Arc::new(data),
            event: Event::new(),
        })?)))
    }

    /// Gets a copy of the current value of the broadcast event.
    pub fn value(&self) -> T {
        (*self.0.lock().data).clone()
    }

    #[inline]
    /// Creates a new listener for the broadcast event.
    pub fn listen(&self) -> BroadcastListener<T> {
        BroadcastListener::new(Weak::new(), Arc::downgrade(&self.0))
    }

    /// Publishes a new value for the broadcast event.
    pub fn publish(&self, data: T) {
        let mut lock = self.0.lock();
        lock.data = Arc::new(data);
        lock.event.notify();
    }
}

#[derive(Clone)]
/// Provides a means of listening to updates from a [`Broadcast`] event.
pub struct BroadcastListener<T: Clone> {
    value: Weak<T>,
    data: Weak<Mutex<BroadcastData<T>>>,
}

impl<T: Clone> BroadcastListener<T> {
    #[inline]
    fn new(value: Weak<T>, data: Weak<Mutex<BroadcastData<T>>>) -> Self {
        Self { value, data }
    }

    #[inline]
    /// Get the latest unprocessed value from the event, if there is one.
    pub fn next_value(&mut self) -> Option<T> {
        Self::next_value_impl(&mut self.value, &self.data)
    }

    #[inline]
    /// A [`Selectable`] event which occurs when new data is published to the
    /// underlying [`Broadcast`] event.
    pub fn select(&'_ mut self) -> impl Selectable<Output = T> + '_ {
        struct BroadcastSelect<'b, T: Clone> {
            value: &'b mut Weak<T>,
            handle: EventHandle<&'b Weak<Mutex<BroadcastData<T>>>>,
        }

        impl<'b, T: Clone> Selectable for BroadcastSelect<'b, T> {
            type Output = T;

            #[inline]
            fn poll(mut self) -> Result<Self::Output, Self> {
                let value = &mut self.value;
                self.handle
                    .with(|data| BroadcastListener::next_value_impl(value, *data))
                    .flatten()
                    .ok_or(self)
            }
            #[inline]
            fn sleep(&self) -> GenericSleep {
                GenericSleep::NotifyTake(None)
            }
        }

        BroadcastSelect {
            value: &mut self.value,
            handle: handle_event(&self.data),
        }
    }

    fn next_value_impl(value: &mut Weak<T>, data: &Weak<Mutex<BroadcastData<T>>>) -> Option<T> {
        let data = data.upgrade()?;
        let lock = data.lock();
        match value.upgrade() {
            Some(arc) if Arc::ptr_eq(&arc, &lock.data) => None,
            _ => {
                *value = Arc::downgrade(&lock.data);
                Some((*lock.data).clone())
            }
        }
    }
}

/// Describes an object which is a source of data, such as a sensor.
///
/// Used to facilitate broadcasting readings via [`IntoBroadcast`].
pub trait DataSource {
    /// The type of data produced by the data source.
    type Data: Clone + 'static;

    /// The type of errors which could occur while reading data.
    type Error;

    /// Tries to read a new value form the data source.
    fn read(&self) -> Result<Self::Data, Self::Error>;
}

/// Extension trait for converting any [`DataSource`] into a
/// [`BroadcastWrapper`] to facilitate broadcasting readings.
pub trait IntoBroadcast: Sized + DataSource {
    /// Converts the data source into a [`BroadcastWrapper`].
    ///
    /// This internally calls [`DataSource::read()`]; if that call fails, the
    /// error is propagated and the data source object is returned.
    fn into_broadcast(self) -> Result<BroadcastWrapper<Self>, (Self::Error, Self)>;
}

impl<T: Sized + DataSource> IntoBroadcast for T {
    fn into_broadcast(self) -> Result<BroadcastWrapper<Self>, (Self::Error, Self)> {
        let data = match self.read() {
            Ok(data) => data,
            Err(err) => return Err((err, self)),
        };

        Ok(BroadcastWrapper {
            inner: self,
            broadcast: Broadcast::new(data),
        })
    }
}

/// Wraps a [`DataSource`], exposing the ability to broadcast readings.
pub struct BroadcastWrapper<T: DataSource> {
    inner: T,
    broadcast: Broadcast<T::Data>,
}

impl<T: DataSource> BroadcastWrapper<T> {
    /// Converts the [`BroadcastWrapper`] back into the internal [`DataSource`].
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Tries to take a new reading and publish it to all listeners.
    pub fn update(&self) -> Result<T::Data, T::Error> {
        let data = self.inner.read()?;
        self.broadcast.publish(data.clone());
        Ok(data)
    }

    /// Creates a [`BroadcastListener`] which receives any new readings.
    pub fn listen(&self) -> BroadcastListener<T::Data> {
        self.broadcast.listen()
    }
}

impl<T: DataSource> Deref for BroadcastWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: DataSource> DerefMut for BroadcastWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> OwnerMut<Event> for &Weak<Mutex<BroadcastData<T>>> {
    fn with<'a, U>(&'a mut self, f: impl FnOnce(&mut Event) -> U) -> Option<U>
    where
        Event: 'a,
    {
        Some(f(&mut self.upgrade()?.try_lock().ok()?.event))
    }
}

struct BroadcastData<T> {
    data: Arc<T>,
    event: Event,
}
