use async_actor::inject::Injector;
use async_actor::inject::assisted_inject::AssistedInstantiable;
use async_actor::inject::singleton::Singleton;
use async_actor::system::Component;
use async_actor_proc::{Component, actor, inject, Singleton, assisted_factory, AssistedInstantiable};

#[tokio::main]
async fn main() {
  let injector = Injector::default().start();
  let some_service = injector.get::<SomeService>().await;
  some_service.do_action().await.print().await;
}


#[derive(Component, Singleton)]
pub struct SomeService {
  #[inject] factory: SomeFactoryImplHandle,
}

#[actor]
impl SomeService {
  async fn do_action(&mut self) -> SomethingHandle {
    self.factory.create("Leo".to_string()).await.start()
  }
}

#[derive(Component, Singleton)]
pub struct SomePrintService {}

#[actor]
impl SomePrintService {
  fn print(&self, name: String) {
    println!("Printing: {}", name);
  }
}

#[assisted_factory]
pub trait SomeFactory {
  async fn create(&self, name: String) -> Something;
}

#[derive(Component, AssistedInstantiable)]
pub struct Something {
  name: String,
  #[inject] some_other_service: SomePrintServiceHandle,
}

#[actor]
impl Something {
  pub async fn print(&self) {
    self.some_other_service.print(self.name.clone()).await;
  }
}