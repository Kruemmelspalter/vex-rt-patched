use alloc::sync::Arc;
use core::time::Duration;
use owner_monad::OwnerMut;

use super::{
    handle_event, Event, EventHandle, GenericSleep, Instant, Mutex, Selectable, Semaphore,
    TIMEOUT_MAX,
};
use crate::error::Error;

/// Represents the sending end of a rendez-vous channel.
pub struct SendChannel<T>(Arc<ChannelShared<T>>);

impl<T> SendChannel<T> {
    /// A [`Selectable`] event which resolves when `value` is sent on the
    /// channel. Respects the atomicity and rendez-vous properties of the
    /// operation; if the event occurs and is processed, then the value was
    /// sent, and otherwise not.
    pub fn select(&self, value: T) -> impl '_ + Selectable<Result = ()> {
        struct SendSelect<'b, T> {
            value: T,
            data: &'b ChannelShared<T>,
        }

        struct SendEvent<'b, T> {
            value: T,
            data: &'b ChannelShared<T>,
            handle: EventHandle<SendWrapper<'b, T>>,
            offset: u32,
        }

        impl<'b, T> Selectable for SendSelect<'b, T> {
            const COUNT: u32 = 1;

            type Result = ();

            type Event = SendEvent<'b, T>;

            fn listen(self, offset: u32) -> Self::Event {
                SendEvent {
                    value: self.value,
                    data: self.data,
                    handle: handle_event(SendWrapper(self.data), offset),
                    offset,
                }
            }

            fn poll(event: Self::Event, _mask: u32) -> Result<(), Self::Event> {
                // Send mutex is locked for the duration of the poll operation.
                let _send_lock = event.data.send_mutex.lock();

                let n = {
                    let mut lock = event.data.data.lock();
                    lock.value = Some(event.value);
                    lock.receive_event.notify();
                    lock.receive_event.task_count()
                };

                // Wait for all receivers to process.
                for _ in 0..n {
                    // TODO: consider shortening this timeout to enforce a realtime guarantee.
                    event
                        .data
                        .ack_sem
                        .wait(Duration::from_millis(TIMEOUT_MAX as u64))
                        .unwrap_or_else(|err| panic!("failed to synchronize on channel: {}", err));
                }

                // Check if the value remains.
                if let Some(value) = event.data.data.lock().value.take() {
                    Err(SendEvent { value, ..event })
                } else {
                    Ok(())
                }
            }

            fn sleep(event: &Self::Event) -> GenericSleep {
                if event.data.data.lock().receive_event.task_count() == 0 {
                    GenericSleep::NotifyTake(None)
                } else {
                    GenericSleep::Timestamp(Instant::from_millis(0), 1u32.rotate_left(event.offset))
                }
            }
        }

        SendSelect {
            value,
            data: &self.0,
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
    pub fn select(&self) -> impl '_ + Selectable<Result = T> {
        struct ReceiveSelect<'b, T> {
            data: &'b ChannelShared<T>,
        }

        struct ReceiveEvent<'b, T> {
            data: &'b ChannelShared<T>,
            handle: EventHandle<ReceiveWrapper<'b, T>>,
            offset: u32,
        }

        impl<'b, T> Selectable for ReceiveSelect<'b, T> {
            const COUNT: u32 = 1;

            type Result = T;

            type Event = ReceiveEvent<'b, T>;

            fn listen(self, offset: u32) -> Self::Event {
                ReceiveEvent {
                    data: self.data,
                    handle: handle_event(ReceiveWrapper(self.data), offset),
                    offset,
                }
            }

            fn poll(event: Self::Event, _mask: u32) -> core::result::Result<T, Self::Event> {
                let mut lock = event.data.data.lock();

                // Ignore failure to post; we don't care.
                event.data.ack_sem.post().unwrap_or(());

                if let Some(value) = lock.value.take() {
                    Ok(value)
                } else {
                    lock.send_event.notify();
                    Err(event)
                }
            }

            fn sleep(event: &Self::Event) -> GenericSleep {
                if event.data.data.lock().send_event.task_count() == 0 {
                    GenericSleep::NotifyTake(None)
                } else {
                    GenericSleep::Timestamp(Instant::from_millis(0), 1u32.rotate_left(event.offset))
                }
            }
        }

        impl<'b, T> Drop for ReceiveSelect<'b, T> {
            fn drop(&mut self) {
                // Keep mutex locked while dropping to avoid race condition.
                let _lock = self.data.data.lock();

                // Ignore failure to post; we don't care.
                self.data.ack_sem.post().unwrap_or(());
            }
        }

        ReceiveSelect { data: &self.0 }
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

impl<'b, T> OwnerMut<Event> for SendWrapper<'b, T> {
    fn with<'a, U>(&'a mut self, f: impl FnOnce(&mut Event) -> U) -> Option<U>
    where
        Event: 'a,
    {
        Some(f(&mut self.0.data.try_lock().ok()?.send_event))
    }
}

struct ReceiveWrapper<'b, T>(&'b ChannelShared<T>);

impl<'b, T> OwnerMut<Event> for ReceiveWrapper<'b, T> {
    fn with<'a, U>(&'a mut self, f: impl FnOnce(&mut Event) -> U) -> Option<U>
    where
        Event: 'a,
    {
        Some(f(&mut self.0.data.try_lock().ok()?.receive_event))
    }
}
