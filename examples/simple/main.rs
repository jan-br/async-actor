use async_actor_proc::{actor, actor_handle, actor_impl};
use async_actor::system::{Component, ComponentMessageHandler, ComponentHandle};

#[tokio::main]
async fn main() {
  //notice some_struct is immutable
  let some_struct = SomeStruct::default().start();
  println!("Modified: {}", some_struct.is_modified().await);

  //some_action is still callable even though it requires &mut self
  some_struct.modify(true).await;
  println!("Modified: {}", some_struct.is_modified().await);
}

#[actor]
#[derive(Default)]
pub struct SomeStruct {
  modified: bool
}

#[actor_impl]
impl SomeStruct {
  #[actor_handle]
  pub async fn modify(&mut self, value: bool) {
    self.modified = value;
  }

  #[actor_handle]
  pub async fn is_modified(&mut self) -> bool {
    self.modified
  }
}