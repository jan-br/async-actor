use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use crate::inject::Injector;
use crate::system::{Component, HasHandleWrapper};

pub trait InjectableInstance where Self: Any + Send + Sync {
  type Inner: HasHandleWrapper<HandleWrapper=Self>;
  fn create_instance(injector: Injector) -> Pin<Box<dyn Future<Output=Box<Self>> + Send + Sync>>;
}

pub trait ManuallyInjectableInstance where Self: Any + Send + Sync {
  type Inner: HasHandleWrapper<HandleWrapper=Self>;
}
impl<T: ManuallyInjectableInstance> InjectableInstance for T {
  type Inner = T::Inner;

  fn create_instance(injector: Injector) -> Pin<Box<dyn Future<Output=Box<Self>> + Send + Sync>> {
    panic!("This struct cannot be injected before it was manually bound.")
  }
}