use async_actor_proc::{actor, Component};
use async_actor::system::{Component};

#[tokio::main]
async fn main() {
  //notice some_struct is immutable
  let some_struct = SomeStruct::default().start();
  println!("Modified: {}", some_struct.is_modified().await);

  //some_action is still callable even though it requires &mut self
  some_struct.modify(true).await;
  println!("Modified: {}", some_struct.is_modified().await);
}


#[derive(Default, Component)]
pub struct SomeStruct {
  modified: bool,
}

#[actor]
impl SomeStruct {
  pub async fn modify(&mut self, value: bool) {
    self.modified = value;
  }

  pub fn is_modified(&mut self) -> bool {
    self.modified
  }
}
