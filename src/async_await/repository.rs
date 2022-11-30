use core::{cell::RefCell, mem::transmute};
use owner_monad::OwnerMut;
use raii_map::map::{insert, Map};
use replace_with::replace_with_or_abort;

use crate::rtos::{select_map, GenericSleep, Selectable};

use super::Promise;

#[derive(Default)]
/// Provides a means of translation between [`Selectable`] events and async
/// futures.
///
/// Proxying an event adds it to the repository and returns a future which
/// resolves with its result. Selecting on the repository selects on all
/// unresolved events, with the result provided by waking the associated future.
///
/// Used internally by [`ExecutionContext`](super::ExecutionContext).
pub struct Repository {
    data: RefCell<Map<*mut dyn SelectProxy, bool>>,
}

impl Repository {
    #[inline]
    /// Creates a new repository.
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    /// A [`Selectable`] event which resolves when any one event in the
    /// repository resolves. The result of the event is provided to the future
    /// which is waiting on it.
    pub fn select(&self) -> impl Selectable<Output = ()> + '_ {
        pub struct RepositorySelect<'a> {
            repo: &'a Repository,
            sleep: GenericSleep,
        }

        impl<'a> Selectable for RepositorySelect<'a> {
            type Output = ();

            #[inline]
            fn poll(mut self) -> Result<Self::Output, Self> {
                self.sleep = GenericSleep::Never;
                let mut data = self.repo.data.borrow_mut();

                for (e, done) in data.iter_mut() {
                    if *done {
                        continue;
                    }

                    let event = unsafe { &mut **e };
                    if event.poll() {
                        *done = true;
                        return Ok(());
                    } else {
                        self.sleep = self.sleep.combine(event.sleep());
                    }
                }

                Err(self)
            }

            #[inline]
            fn sleep(&self) -> GenericSleep {
                self.sleep
            }
        }

        RepositorySelect {
            repo: self,
            sleep: GenericSleep::Ready,
        }
    }

    #[inline]
    /// Adds an event to the repository and returns a future which resolves with
    /// its result. If the future is dropped, the event is automatically removed
    /// from the repository and dropped.
    pub async fn proxy<'a, T: 'a>(&'a self, event: impl Selectable<Output = T> + 'a) -> T {
        let r = RepoRef(self);
        let (promise, resolve) = Promise::new();
        let mut proxy = Proxy(Some(select_map(event, resolve)));
        let ptr: *mut (dyn SelectProxy + 'a) = &mut proxy;
        let _handle = insert(r, unsafe { transmute(ptr) }, false).unwrap();
        promise.await
    }
}

struct Proxy<E: Selectable<Output = ()>>(Option<E>);

impl<E: Selectable<Output = ()>> SelectProxy for Proxy<E> {
    fn poll(&mut self) -> bool {
        replace_with_or_abort(&mut self.0, |e| e?.poll().err());
        self.0.is_none()
    }

    fn sleep(&self) -> GenericSleep {
        self.0.as_ref().expect("already done").sleep()
    }
}

struct RepoRef<'a>(&'a Repository);

impl<'a> OwnerMut<Map<*mut dyn SelectProxy, bool>> for RepoRef<'a> {
    fn with<'b, U>(
        &'b mut self,
        f: impl FnOnce(&mut Map<*mut dyn SelectProxy, bool>) -> U,
    ) -> Option<U>
    where
        Map<*mut dyn SelectProxy, bool>: 'b,
    {
        self.0.data.try_borrow_mut().ok().map(|mut r| f(&mut r))
    }
}

trait SelectProxy {
    fn poll(&mut self) -> bool;

    fn sleep(&self) -> GenericSleep;
}
