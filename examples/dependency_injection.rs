use async_actor::inject::{Injector};
use async_actor::inject::singleton::{Singleton};
use async_actor_proc::{actor, Component, inject, Singleton};


#[tokio::main]
async fn main() {
  // Instantiate Injector
  let injector = Injector::default();
  // Get actor handle of EntryPoint
  let entry_point: EntryPointHandle = injector.get::<EntryPoint>().await;
  // Execute run action on actor
  entry_point.run().await;
}

#[derive(Component, Singleton)]
pub struct EntryPoint {
  #[inject] user_service: UserServiceHandle,
}

#[actor]
impl EntryPoint {
  pub async fn run(&mut self) {
    // Notify UserService actor about joined user Jan
    self.user_service.user_joined("Jan".to_string()).await;
    // Notify UserService actor about joined user Paskal
    self.user_service.user_joined("Paskal".to_string()).await;
  }
}

#[derive(Component, Singleton)]
pub struct UserService {
  #[inject] database_service: DatabaseServiceHandle,
}

#[actor]
impl UserService {
  pub async fn user_joined(&mut self, user: String) {
    self.database_service.save_user(user).await;
  }
}


#[derive(Component, Singleton)]
pub struct DatabaseService {
  connected: bool,
}

#[actor]
impl DatabaseService {
  pub async fn establish_connection(&mut self) {
    self.connected = true;
    println!("Database connection successfully established");
  }

  pub async fn is_connected(&mut self) -> bool{
    self.connected
  }

  pub async fn save_user(&mut self, user: String) {
    // Ensure service is connected to database
    if !self.is_connected().await {
      println!("Database connection not yet established.. Starting connection process..");
      self.establish_connection().await;
    } else {
      println!("Database connection already established");
    }

    println!("Saved user: {}", user);
  }
}


