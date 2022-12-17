use std::future::Future;
use std::pin::Pin;
use tokio::sync::{Mutex, OnceCell};

pub struct LazyCell<T> {
  cell: OnceCell<T>,
  future: Mutex<Option<Pin<Box<dyn Future<Output=T> + Send + Sync>>>>,
}

impl<T> LazyCell<T> {
  pub async fn get(&self) -> &T {
    let mut future_generator_guard = self.future.lock().await;
    let future_generator = future_generator_guard.take().unwrap_or_else(|| Box::pin(async move { unreachable!() }));
    self.cell.get_or_init(||future_generator).await
  }

  pub fn new(future: impl Future<Output=T> + Send + Sync + 'static ) -> Self {
    Self {
      cell: Default::default(),
      future: Mutex::new(Some(Box::pin(future)))
    }
  }
}