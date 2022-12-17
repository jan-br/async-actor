use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use crate::inject::Injector;
use crate::system::{Component, HasHandleWrapper};

pub trait Singleton where Self: Any + Send + Sync + Clone {
  type Inner: HasHandleWrapper<HandleWrapper=Self>;
  fn create_instance(injector: Injector) -> Pin<Box<dyn Future<Output=Self::Inner> + Send + Sync>>;
}