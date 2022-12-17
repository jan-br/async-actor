use std::future::Future;
use std::sync::Arc;
use tokio::sync::{Mutex};
use tokio::sync::oneshot::{channel, Receiver, Sender};

struct LazyInner<T: Clone> {
  value: Option<T>,
  receivers: Vec<Sender<T>>,
}

impl<T: Clone> LazyInner<T> {
  fn get(&mut self) -> Receiver<T> {
    let (sender, receiver) = channel();
    self.receivers.push(sender);
    if let Some(value) = self.value.clone() {
      for sender in std::mem::take(&mut self.receivers) {
        sender.send(value.clone()).map_err(|_| ()).unwrap();
      }
    }
    receiver
  }
}

#[derive(Clone)]
pub struct Lazy<T: Clone> {
  inner: Arc<Mutex<LazyInner<T>>>,
}

impl<T: Clone + Send + Sync + 'static> Lazy<T> {
  pub async fn get(&self) -> T {
    let mut guard = self.inner.lock().await;
    let receiver = guard.get();
    drop(guard);
    receiver.await.unwrap()
  }

  pub fn run(future: impl Future<Output=T> + Send + Sync + 'static) -> Self {
    let lazy = Lazy { inner: Arc::new(Mutex::new(LazyInner { value: None, receivers: vec![] })) };
    let future = Box::pin(future);
    tokio::spawn({
      let lazy = lazy.clone();
      async move {
        let lazy = lazy;
        let mut inner = lazy.inner.lock().await;
        let value = future.await;
        inner.value = Some(value.clone());
        for sender in std::mem::take(&mut inner.receivers) {
          sender.send(value.clone()).map_err(|_| ()).unwrap();
        }
      }
    });
    lazy
  }
}
