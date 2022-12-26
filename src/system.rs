use crate::util::container::Container;
use crate::util::resolvable::{AsyncResolvable, Resolver, SyncResolvable};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[repr(transparent)]
pub struct SendVoidPtr(pub *mut std::ffi::c_void);

unsafe impl Send for SendVoidPtr {}

pub trait HasHandleWrapper {
  type HandleWrapper: Clone + Send + Sync + 'static;
}

pub trait PostConstruct {
  fn init<'a>(&'a mut self) -> Pin<Box<dyn Future<Output=()> + Send + Sync + 'a>>;
}

default impl<T> PostConstruct for T {
  fn init<'a>(&'a mut self) -> Pin<Box<dyn Future<Output=()> + Send + Sync + 'a>> {
    Box::pin(async move {})
  }
}

#[async_trait::async_trait]
pub trait Component: HasHandleWrapper + Sized + Send + 'static {
  fn create_wrapper(handle: ComponentHandle<Self>) -> Self::HandleWrapper;

  fn start(self) -> Self::HandleWrapper {
    let (receiver, handle) = ComponentHandle::create();
    let wrapper = Self::create_wrapper(handle);

    let runner = DefaultComponentRunner::run(self, receiver);

    tokio::spawn(runner);

    wrapper
  }
}

type PinnedFuture<'a> = Pin<Box<dyn Future<Output=()> + Send + 'a>>;
type ComponentMessageDispatchFn<C> =
fn(&mut C, SendVoidPtr) -> PinnedFuture<'_>;

#[async_trait::async_trait]
pub trait ComponentMessageHandler<R>
  where
    Self: 'static + Component,
    R: 'static + Send,
{
  type Answer: 'static + Send;

  fn dispatch(&mut self, payload: SendVoidPtr) -> PinnedFuture {
    let resolver =
      unsafe { Container::<Resolver<R, Self::Answer>>::from_raw(payload.0) }.into_inner();

    Box::pin(async move {
      let (resolver, meta) = resolver.split();
      let answer = self.handle(meta).await;
      resolver.resolve(answer);
    })
  }

  async fn handle(&mut self, request: R) -> Self::Answer;
}

struct AnyComponentMessage<C>
  where
    C: Component,
{
  payload: *mut std::ffi::c_void,
  dispatcher: ComponentMessageDispatchFn<C>,
}

unsafe impl<C> Send for AnyComponentMessage<C> where C: Component {}

#[derive(Debug)]
pub struct ComponentHandle<C>
  where
    C: Component,
{
  sender: UnboundedSender<AnyComponentMessage<C>>,
}

impl<C> ComponentHandle<C>
  where
    C: Component,
{
  fn new(sender: UnboundedSender<AnyComponentMessage<C>>) -> Self {
    Self { sender }
  }

  fn create() -> (UnboundedReceiver<AnyComponentMessage<C>>, Self) {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();

    (receiver, Self::new(sender))
  }

  pub async fn dispatch<M>(&self, message: M) -> <C as ComponentMessageHandler<M>>::Answer
    where
      C: ComponentMessageHandler<M>,
      M: Send + 'static,
  {
    DispatcherImpl::dispatch_async(&self.sender, message).await
  }

  pub fn dispatch_sync_nowait<M>(&self, message: M)
    where
      C: ComponentMessageHandler<M>,
      M: Send + 'static,
  {
    DispatcherImpl::dispatch_sync_nowait(&self.sender, message)
  }

  pub fn dispatch_sync<M>(&self, message: M) -> C::Answer
    where
      C: ComponentMessageHandler<M>,
      M: Send + 'static,
  {
    DispatcherImpl::dispatch_sync(&self.sender, message)
  }

  pub fn make_sender<M>(&self) -> MessageSender<M, <C as ComponentMessageHandler<M>>::Answer>
    where
      C: ComponentMessageHandler<M>,
      M: Send + 'static,
  {
    MessageSender::<M, C::Answer>::create(self.sender.clone())
  }

  pub fn make_transforming_sender<M, N, T>(
    &self,
    transformer: T,
  ) -> MessageSender<M, <C as ComponentMessageHandler<N>>::Answer>
    where
      C: ComponentMessageHandler<N>,
      M: Send + 'static,
      N: Send + 'static,
      T: Fn(M) -> N + Send + Sync + 'static,
  {
    MessageSender::<M, C::Answer>::create_transforming(self.sender.clone(), transformer)
  }
}

impl<C> Clone for ComponentHandle<C>
  where
    C: Component,
{
  fn clone(&self) -> Self {
    let sender = self.sender.clone();

    Self { sender }
  }
}

impl<C> From<ComponentHandle<C>> for ComponentHandleUnique<C> where C: Component {
  fn from(value: ComponentHandle<C>) -> Self {
    Self {
      sender: value.sender.clone()
    }
  }
}

#[derive(Debug)]
pub struct ComponentHandleUnique<C>
  where
    C: Component,
{
  sender: UnboundedSender<AnyComponentMessage<C>>,
}

impl<C> ComponentHandleUnique<C>
  where
    C: Component,
{
  fn new(sender: UnboundedSender<AnyComponentMessage<C>>) -> Self {
    Self { sender }
  }

  fn create() -> (UnboundedReceiver<AnyComponentMessage<C>>, Self) {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();

    (receiver, Self::new(sender))
  }

  pub async fn dispatch<M>(&self, message: M) -> <C as ComponentMessageHandler<M>>::Answer
    where
      C: ComponentMessageHandler<M>,
      M: Send + 'static,
  {
    DispatcherImpl::dispatch_async(&self.sender, message).await
  }

  pub fn dispatch_sync_nowait<M>(&self, message: M)
    where
      C: ComponentMessageHandler<M>,
      M: Send + 'static,
  {
    DispatcherImpl::dispatch_sync_nowait(&self.sender, message)
  }

  pub fn dispatch_sync<M>(&self, message: M) -> C::Answer
    where
      C: ComponentMessageHandler<M>,
      M: Send + 'static,
  {
    DispatcherImpl::dispatch_sync(&self.sender, message)
  }

  pub fn make_sender<M>(&self) -> MessageSender<M, <C as ComponentMessageHandler<M>>::Answer>
    where
      C: ComponentMessageHandler<M>,
      M: Send + 'static,
  {
    MessageSender::<M, C::Answer>::create(self.sender.clone())
  }

  pub fn make_transforming_sender<M, N, T>(
    &self,
    transformer: T,
  ) -> MessageSender<M, <C as ComponentMessageHandler<N>>::Answer>
    where
      C: ComponentMessageHandler<N>,
      M: Send + 'static,
      N: Send + 'static,
      T: Fn(M) -> N + Send + Sync + 'static,
  {
    MessageSender::<M, C::Answer>::create_transforming(self.sender.clone(), transformer)
  }
}


type FutureMessageDispatcher<M, R> =
Arc<dyn Fn(M) -> Pin<Box<dyn Future<Output=R> + Send>> + Send + Sync>;
type SyncNowaitMessageDispatcher<M> = Arc<dyn Fn(M) + Send + Sync>;
type SyncMessageDispatcher<M, R> = Arc<dyn Fn(M) -> R + Send + Sync>;

#[derive(Clone)]
pub struct MessageSender<M, R> {
  async_dispatcher: FutureMessageDispatcher<M, R>,
  sync_nowait_dispatcher: SyncNowaitMessageDispatcher<M>,
  sync_dispatcher: SyncMessageDispatcher<M, R>,
}

impl<M, R> MessageSender<M, R> {
  fn create<C>(sender: UnboundedSender<AnyComponentMessage<C>>) -> Self
    where
      C: ComponentMessageHandler<M, Answer=R>,
      M: Send + 'static,
      R: Send,
  {
    let fut_sender = sender.clone();
    let async_dispatcher =
      move |message: M| -> Pin<Box<dyn Future<Output=C::Answer> + Send>> {
        let fut_sender = fut_sender.clone();

        Box::pin(async move { DispatcherImpl::dispatch_async(&fut_sender, message).await })
      };

    let nowait_sender = sender.clone();
    let sync_nowait_dispatcher =
      move |message: M| DispatcherImpl::dispatch_sync_nowait(&nowait_sender, message);

    let sync_dispatcher = move |message: M| DispatcherImpl::dispatch_sync(&sender, message);

    Self {
      async_dispatcher: Arc::new(async_dispatcher),
      sync_nowait_dispatcher: Arc::new(sync_nowait_dispatcher),
      sync_dispatcher: Arc::new(sync_dispatcher),
    }
  }

  fn create_transforming<C, N, T>(
    sender: UnboundedSender<AnyComponentMessage<C>>,
    transformer: T,
  ) -> Self
    where
      C: ComponentMessageHandler<N, Answer=R>,
      M: Send + 'static,
      N: Send + 'static,
      R: Send,
      T: Fn(M) -> N + Send + Sync + 'static,
  {
    let fut_sender = sender.clone();
    let transformer = Arc::new(transformer);

    let transformer_x = transformer.clone();
    let async_dispatcher =
      move |message: M| -> Pin<Box<dyn Future<Output=C::Answer> + Send>> {
        let fut_sender = fut_sender.clone();
        let transformer = transformer_x.clone();

        let message = (*transformer)(message);

        Box::pin(async move { DispatcherImpl::dispatch_async(&fut_sender, message).await })
      };

    let nowait_sender = sender.clone();

    let transformer_x = transformer.clone();
    let sync_nowait_dispatcher = move |message: M| {
      let transformer = transformer_x.clone();
      let message = (*transformer)(message);

      DispatcherImpl::dispatch_sync_nowait(&nowait_sender, message);
    };

    let sync_dispatcher = move |message: M| {
      let message = (*transformer)(message);

      DispatcherImpl::dispatch_sync(&sender, message)
    };

    Self {
      async_dispatcher: Arc::new(async_dispatcher),
      sync_nowait_dispatcher: Arc::new(sync_nowait_dispatcher),
      sync_dispatcher: Arc::new(sync_dispatcher),
    }
  }

  pub async fn dispatch(&self, message: M) -> R {
    (self.async_dispatcher)(message).await
  }

  pub fn dispatch_sync_nowait(&self, message: M) {
    (self.sync_nowait_dispatcher)(message)
  }

  pub fn dispatch_sync(&self, message: M) -> R {
    (self.sync_dispatcher)(message)
  }
}

struct DispatcherImpl;

impl DispatcherImpl {
  async fn dispatch_async<C, M>(
    sender: &UnboundedSender<AnyComponentMessage<C>>,
    message: M,
  ) -> C::Answer
    where
      C: ComponentMessageHandler<M>,
      M: Send + 'static,
  {
    let (resolvable, resolver) = AsyncResolvable::new_with_meta(message);
    let message = Self::make_message(resolver);

    let _ = sender.send(message);
    resolvable.await.unwrap()
  }

  fn dispatch_sync_nowait<C, M>(sender: &UnboundedSender<AnyComponentMessage<C>>, message: M)
    where
      C: ComponentMessageHandler<M>,
      M: Send + 'static,
  {
    let resolver = Resolver::<M, C::Answer>::noop(message);
    let message = Self::make_message(resolver);
    let _ = sender.send(message);
  }

  fn dispatch_sync<C, M>(
    sender: &UnboundedSender<AnyComponentMessage<C>>,
    message: M,
  ) -> C::Answer
    where
      C: ComponentMessageHandler<M>,
      M: Send + 'static,
  {
    let (resolvable, resolver) = SyncResolvable::new_with_meta(message);
    let message = Self::make_message(resolver);
    let _ = sender.send(message);

    resolvable.wait().unwrap()
  }

  fn make_message<C, M, P>(payload: P) -> AnyComponentMessage<C>
    where
      C: ComponentMessageHandler<M>,
      M: Send + 'static,
  {
    AnyComponentMessage {
      payload: Container::new(payload).into_raw(),
      dispatcher: |component, data| {
        <C as ComponentMessageHandler<M>>::dispatch(component, data)
      },
    }
  }
}

pub struct DefaultComponentRunner<C>(C)
  where
    C: Component;

impl<C> DefaultComponentRunner<C>
  where
    C: Component,
{
  async fn run(
    mut component: C,
    mut receiver: UnboundedReceiver<AnyComponentMessage<C>>,
  ) {
    while let Some(message) = receiver.recv().await {
      let dispatcher = message.dispatcher;
      let payload = SendVoidPtr(message.payload);

      (dispatcher)(&mut component, payload).await;
    }
  }
}

pub trait EnsureNotDroppedForDuration {
  fn ensure_not_dropped_for_duration(self: &Arc<Self>, duration: Duration) -> Pin<Box<dyn Fn() + Send + Sync>>;
}