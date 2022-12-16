use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use crate::inject::Injector;
use crate::system::Component;

pub trait Singleton where Self: Any + Send + Sync + Clone {
  type Component: Component<HandleWrapper=Self>;
  fn create(injector: Injector) -> Pin<Box<dyn Future<Output=Self::Component> + Send + Sync>>;
}