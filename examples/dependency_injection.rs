use std::future::Future;
use std::pin::Pin;
use async_actor::inject::{Injector};
use async_actor::inject::singleton::{Singleton};
use async_actor_proc::{actor, actor_handle, actor_impl, inject, Singleton};
use async_actor::system::{Component, ComponentMessageHandler, ComponentHandle};


#[tokio::main]
async fn main() {
  // Instantiate Injector
  let injector = Injector::default();
  // Get actor handle of EntryPoint
  let entry_point: EntryPointHandle = injector.get::<EntryPoint>().await;
  // Execute run action on actor
  entry_point.run().await;
}

#[actor]
#[derive(Singleton)]
pub struct EntryPoint {
  #[inject] user_service: UserServiceHandle,
}

#[actor_impl]
impl EntryPoint {
  #[actor_handle]
  pub async fn run(&mut self) {
    // Notify UserService actor about joined user Jan
    self.user_service.user_joined("Jan".to_string()).await;
    // Notify UserService actor about joined user Paskal
    self.user_service.user_joined("Paskal".to_string()).await;
  }
}

#[actor]
#[derive(Singleton)]
pub struct UserService {
  #[inject] database_service: DatabaseServiceHandle,
}

#[actor_impl]
impl UserService {
  #[actor_handle]
  pub async fn user_joined(&mut self, user: String) {
    self.database_service.save_user(user).await;
  }
}


#[actor]
#[derive(Singleton)]
pub struct DatabaseService {
  connected: bool,
}

#[actor_impl]
impl DatabaseService {
  #[actor_handle]
  pub async fn establish_connection(&mut self) {
    self.connected = true;
    println!("Database connection successfully established");
  }

  #[actor_handle]
  pub async fn is_connected(&mut self) -> bool{
    self.connected
  }

  #[actor_handle]
  pub async fn save_user(&mut self, user: String) {
    // Ensure service is connected to database
    // ** Access the current actor handle with hidden variable `wrapper` **
    if !wrapper.is_connected().await {
      println!("Database connection not yet established.. Starting connection process..");
      wrapper.establish_connection().await;
    } else {
      println!("Database connection already established");
    }

    println!("Saved user: {}", user);
  }
}


