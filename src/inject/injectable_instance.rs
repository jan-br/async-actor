use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use crate::inject::Injector;
use crate::system::{Component, HasHandleWrapper};

pub trait InjectableInstance where Self: Any + Send + Sync {
  type Inner: HasHandleWrapper<HandleWrapper=Self>;
  fn create_instance(injector: Injector) -> Pin<Box<dyn Future<Output=Box<Self>> + Send + Sync>>;
}