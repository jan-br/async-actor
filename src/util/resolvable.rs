use crate::util::container::Container;
use crate::util::debug::PointerDebug;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct Resolver<M, T> {
    meta: M,
    data: *mut std::ffi::c_void,
    resolver: Option<fn(data: *mut std::ffi::c_void, value: Option<T>)>,
}

impl<M, T> Resolver<M, T> {
    pub fn noop(meta: M) -> Self {
        Self {
            meta,
            data: std::ptr::null_mut(),
            resolver: Some(Self::noop_resolve),
        }
    }

    fn noop_resolve(_data: *mut std::ffi::c_void, _value: Option<T>) {}

    pub fn get_meta(&self) -> &M {
        &self.meta
    }

    pub fn resolve(mut self, value: T) {
        (self.resolver.take().unwrap())(self.data, Some(value));
    }

    pub fn split(self) -> (ThinResolver<T>, M) {
        let thin = ThinResolver {
            data: self.data,
            resolver: self.resolver.unwrap(),
        };
        let meta = unsafe { std::ptr::read(&self.meta) };

        std::mem::forget(self);

        (thin, meta)
    }
}

impl<M, T> Drop for Resolver<M, T> {
    fn drop(&mut self) {
        if let Some(resolver) = self.resolver.take() {
            (resolver)(self.data, None);
        }
    }
}

impl<M, T> Debug for Resolver<M, T>
where
    M: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncResolver")
            .field("meta", &self.meta)
            .field("data", &PointerDebug::new(self.data))
            .field(
                "resolver",
                &PointerDebug::new(self.resolver.unwrap() as *const ()),
            )
            .finish()
    }
}

unsafe impl<M, T> Send for Resolver<M, T>
where
    M: Send,
    T: Send,
{
}

pub struct ThinResolver<T> {
    data: *mut std::ffi::c_void,
    resolver: fn(data: *mut std::ffi::c_void, value: Option<T>),
}

impl<T> ThinResolver<T> {
    pub fn resolve(self, value: T) {
        let data = self.data;
        let resolver = self.resolver;

        std::mem::forget(self);

        (resolver)(data, Some(value));
    }
}

impl<T> Drop for ThinResolver<T> {
    fn drop(&mut self) {
        (self.resolver)(self.data, None);
    }
}

unsafe impl<T> Send for ThinResolver<T> where T: Send {}

impl<T> Debug for ThinResolver<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThinResolver")
            .field("data", &PointerDebug::new(self.data))
            .field("resolver", &PointerDebug::new(self.resolver as *const ()))
            .finish()
    }
}

struct TokioResolver;

impl TokioResolver {
    fn resolve<T>(data: *mut std::ffi::c_void, value: Option<T>) {
        let container = unsafe { Container::<tokio::sync::oneshot::Sender<T>>::from_raw(data) };

        if let Some(value) = value {
            drop(container.into_inner().send(value));
        }
    }
}

#[pin_project::pin_project]
#[derive(Debug)]
pub struct AsyncResolvable<T> {
    #[pin]
    receiver: tokio::sync::oneshot::Receiver<T>,
}

impl<T> AsyncResolvable<T> {
    pub fn new() -> (AsyncResolvable<T>, Resolver<(), T>) {
        Self::new_with_meta(())
    }

    pub fn new_with_meta<M>(meta: M) -> (AsyncResolvable<T>, Resolver<M, T>) {
        let (sender, receiver) = tokio::sync::oneshot::channel();

        let container = Container::new(sender);

        let resolvable = AsyncResolvable { receiver };
        let resolver = Resolver {
            meta,
            data: container.into_raw(),
            resolver: Some(TokioResolver::resolve::<T>),
        };

        (resolvable, resolver)
    }
}

impl<T> Future for AsyncResolvable<T> {
    type Output = Result<T, tokio::sync::oneshot::error::RecvError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().receiver.poll(cx)
    }
}

#[derive(Debug)]
pub struct SyncResolvable<T> {
    receiver: tokio::sync::oneshot::Receiver<T>,
}

impl<T> SyncResolvable<T> {
    pub fn new() -> (SyncResolvable<T>, Resolver<(), T>) {
        Self::new_with_meta(())
    }

    pub fn new_with_meta<M>(meta: M) -> (SyncResolvable<T>, Resolver<M, T>) {
        let (sender, receiver) = tokio::sync::oneshot::channel();

        let container = Container::new(sender);

        let resolvable = SyncResolvable { receiver };
        let resolver = Resolver {
            meta,
            data: container.into_raw(),
            resolver: Some(TokioResolver::resolve::<T>),
        };

        (resolvable, resolver)
    }

    pub fn wait(self) -> Option<T> {
        self.receiver.blocking_recv().ok()
    }
}
