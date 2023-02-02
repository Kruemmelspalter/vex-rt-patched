use alloc::sync::Arc;
use owner_monad::OwnerMut;
use queue_model::QueueModel;

use super::{handle_event, Event, EventHandle, GenericSleep, Mutex, Selectable};
use crate::error::Error;

#[repr(transparent)]
/// Represents the sending end of a message-passing queue.
pub struct SendQueue<T>(Arc<dyn QueueShared<T> + Send + Sync>);

impl<T> SendQueue<T> {
    #[inline]
    /// Attempts to send an item on a queue.
    pub fn send(&self, item: T) -> bool {
        self.0.send(item)
    }
}

impl<T> Clone for SendQueue<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[repr(transparent)]
/// Represents the receive end of a message-passing queue.
pub struct ReceiveQueue<T>(Arc<dyn QueueShared<T> + Send + Sync>);

impl<T> ReceiveQueue<T> {
    /// A [`Selectable`] event which resolves when a value is received on the
    /// message-passing queue.
    pub fn select(&self) -> impl '_ + Selectable<Output = T> {
        struct ReceiveSelect<'b, T> {
            data: &'b dyn QueueShared<T>,
            _handle: EventHandle<ReceiveWrapper<'b, T>>,
        }

        impl<'b, T> Selectable for ReceiveSelect<'b, T> {
            type Output = T;

            fn poll(self) -> Result<Self::Output, Self> {
                self.data.receive().ok_or(self)
            }

            fn sleep(&self) -> GenericSleep {
                if self.data.is_empty() {
                    GenericSleep::NotifyTake(None)
                } else {
                    GenericSleep::Ready
                }
            }
        }

        ReceiveSelect {
            data: &*self.0,
            _handle: handle_event(ReceiveWrapper(&*self.0)),
        }
    }
}

impl<T> Clone for ReceiveQueue<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// The send/receive pair type returned by [`queue()`] and [`try_queue()`] for a
/// given queue type.
pub type QueuePair<Q> = (
    SendQueue<<Q as QueueModel>::Item>,
    ReceiveQueue<<Q as QueueModel>::Item>,
);

#[inline]
/// Creates a new send-receive pair together representing a message-passing
/// queue, based on the given underlying queue structure. Panics on failure; see
/// [`try_queue`].
pub fn queue<Q: 'static + QueueModel + Send + Sync>(queue: Q) -> QueuePair<Q> {
    try_queue(queue).unwrap_or_else(|err| panic!("failed to create channel: {}", err))
}

/// Creates a new send-receive pair together representing a message-passing
/// queue, based on the given underlying queue structure.
pub fn try_queue<Q: 'static + QueueModel + Send + Sync>(queue: Q) -> Result<QueuePair<Q>, Error> {
    #[repr(transparent)]
    struct Queue<Q: QueueModel>(Mutex<QueueData<Q>>);

    impl<Q: QueueModel> QueueShared<Q::Item> for Queue<Q> {
        fn send(&self, item: Q::Item) -> bool {
            let mut lock = self.0.lock();

            if lock.queue.enqueue(item) {
                lock.event.notify();
                true
            } else {
                false
            }
        }

        fn receive(&self) -> Option<Q::Item> {
            self.0.lock().queue.dequeue()
        }

        fn is_empty(&self) -> bool {
            self.0.lock().queue.is_empty()
        }

        fn with_event<'a>(&'a self, f: &'a mut dyn FnMut(&mut Event)) {
            f(&mut self.0.lock().event);
        }
    }

    struct QueueData<Q: QueueModel> {
        event: Event,
        queue: Q,
    }

    let data = Arc::new(Queue(Mutex::try_new(QueueData {
        event: Event::new(),
        queue,
    })?));
    let send = SendQueue(data.clone());
    let receive = ReceiveQueue(data);
    Ok((send, receive))
}

trait QueueShared<T> {
    fn send(&self, item: T) -> bool;
    fn receive(&self) -> Option<T>;
    fn is_empty(&self) -> bool;
    fn with_event<'a>(&'a self, f: &'a mut dyn FnMut(&mut Event));
}

#[repr(transparent)]
struct ReceiveWrapper<'b, T>(&'b dyn QueueShared<T>);

impl<'b, T> OwnerMut<Event> for ReceiveWrapper<'b, T> {
    fn with<'a, U>(&'a mut self, f: impl FnOnce(&mut Event) -> U) -> Option<U>
    where
        Event: 'a,
    {
        let mut f = Some(f);
        let mut out: Option<U> = None;
        self.0.with_event(&mut |e| out = Some(f.take().unwrap()(e)));
        out
    }
}
