use alloc::sync::Arc;
use core::time::Duration;

use super::{
    handle_event, Event, EventHandle, GenericSleep, Instant, Mutex, Selectable, Semaphore,
    TIMEOUT_MAX,
};
use crate::{error::Error, util::owner::Owner};

/// Represents the sending end of a rendez-vous channel.
pub struct SendChannel<T>(Arc<ChannelShared<T>>);

impl<T> SendChannel<T> {
    /// A [`Selectable`] event which resolves when `value` is sent on the
    /// channel. Respects the atomicity and rendez-vous properties of the
    /// operation; if the event occurs and is processed, then the value was
    /// sent, and otherwise not.
    pub fn select(&self, value: T) -> impl '_ + Selectable {
        struct SendSelect<'b, T> {
            value: T,
            data: &'b ChannelShared<T>,
            handle: EventHandle<SendWrapper<'b, T>>,
        }

        impl<'b, T> Selectable for SendSelect<'b, T> {
            fn poll(self) -> Result<(), Self> {
                // Send mutex is locked for the duration of the poll operation.
                let _send_lock = self.data.send_mutex.lock();

                let n = {
                    let mut lock = self.data.data.lock();
                    lock.value = Some(self.value);
                    lock.receive_event.notify();
                    lock.receive_event.task_count()
                };

                // Wait for all receivers to process.
                for _ in 0..n {
                    // TODO: consider shortening this timeout to enforce a realtime guarantee.
                    self.data
                        .ack_sem
                        .wait(Duration::from_millis(TIMEOUT_MAX as u64))
                        .unwrap_or_else(|err| panic!("failed to synchronize on channel: {}", err));
                }

                // Check if the value remains.
                if let Some(value) = self.data.data.lock().value.take() {
                    Err(Self {
                        value,
                        data: self.data,
                        handle: self.handle,
                    })
                } else {
                    Ok(())
                }
            }

            fn sleep(&self) -> GenericSleep {
                if self.data.data.lock().receive_event.task_count() == 0 {
                    GenericSleep::NotifyTake(None)
                } else {
                    GenericSleep::Timestamp(Instant::from_millis(0))
                }
            }
        }

        SendSelect {
            value,
            data: &self.0,
            handle: handle_event(SendWrapper(&*self.0)),
        }
    }
}

impl<T> Clone for SendChannel<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// Represents the receive end of a rendez-vous channel.
pub struct ReceiveChannel<T>(Arc<ChannelShared<T>>);

impl<T> ReceiveChannel<T> {
    /// A [`Selectable`] event which resolves when a value is received on the
    /// channel.
    pub fn select(&self) -> impl '_ + Selectable<T> {
        struct ReceiveSelect<'b, T> {
            data: &'b ChannelShared<T>,
            handle: EventHandle<ReceiveWrapper<'b, T>>,
        }

        impl<'b, T> Selectable<T> for ReceiveSelect<'b, T> {
            fn poll(self) -> core::result::Result<T, Self> {
                let mut lock = self.data.data.lock();

                // Ignore failure to post; we don't care.
                self.data.ack_sem.post().unwrap_or(());

                if let Some(value) = lock.value.take() {
                    Ok(value)
                } else {
                    lock.send_event.notify();
                    Err(self)
                }
            }

            fn sleep(&self) -> GenericSleep {
                if self.data.data.lock().send_event.task_count() == 0 {
                    GenericSleep::NotifyTake(None)
                } else {
                    GenericSleep::Timestamp(Instant::from_millis(0))
                }
            }
        }

        impl<'b, T> Drop for ReceiveSelect<'b, T> {
            fn drop(&mut self) {
                // Keep mutex locked while dropping to avoid race condition.
                let _lock = self.data.data.lock();

                // Ignore failure to post; we don't care.
                self.data.ack_sem.post().unwrap_or(());
                self.handle.clear();
            }
        }

        ReceiveSelect {
            data: &self.0,
            handle: handle_event(ReceiveWrapper(&*self.0)),
        }
    }
}

impl<T> Clone for ReceiveChannel<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// Creates a new send-receive pair together representing a rendez-vous channel.
/// Panics on failure; see [`try_channel`].
pub fn channel<T>() -> (SendChannel<T>, ReceiveChannel<T>) {
    try_channel().unwrap_or_else(|err| panic!("failed to create channel: {}", err))
}

/// Creates a new send-receive pair together representing a rendez-vous channel.
pub fn try_channel<T>() -> Result<(SendChannel<T>, ReceiveChannel<T>), Error> {
    let data = Arc::new(ChannelShared {
        data: Mutex::try_new(ChannelData {
            send_event: Event::new(),
            receive_event: Event::new(),
            value: None,
        })?,
        send_mutex: Mutex::try_new(())?,
        ack_sem: Semaphore::try_new(u32::MAX, 0)?,
    });
    let send = SendChannel(data.clone());
    let receive = ReceiveChannel(data);
    Ok((send, receive))
}

struct ChannelShared<T> {
    data: Mutex<ChannelData<T>>,
    send_mutex: Mutex<()>,
    ack_sem: Semaphore,
}

struct ChannelData<T> {
    send_event: Event,
    receive_event: Event,
    value: Option<T>,
}

struct SendWrapper<'b, T>(&'b ChannelShared<T>);

impl<'b, T> Owner<Event> for SendWrapper<'b, T> {
    fn with<U>(&self, f: impl for<'a> FnOnce(&'a mut Event) -> U) -> Option<U> {
        Some(f(&mut self.0.data.try_lock().ok()?.send_event))
    }
}

struct ReceiveWrapper<'b, T>(&'b ChannelShared<T>);

impl<'b, T> Owner<Event> for ReceiveWrapper<'b, T> {
    fn with<U>(&self, f: impl for<'a> FnOnce(&'a mut Event) -> U) -> Option<U> {
        Some(f(&mut self.0.data.try_lock().ok()?.receive_event))
    }
}
