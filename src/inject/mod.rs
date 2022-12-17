use std::any::{Any, type_name, TypeId};
use std::collections::{HashMap};
use std::future::Future;
use std::ops::Deref;
use std::sync::Arc;
use async_lock::{RwLock, RwLockUpgradableReadGuard};
use async_actor_proc::{actor, Component, Injectable};
use crate as async_actor;
use crate::inject::injectable_instance::InjectableInstance;
use crate::system::{Component, ComponentMessageHandler, ComponentHandle, HasHandleWrapper};
use crate::util::lazy::Lazy;

pub mod injectable_instance;
pub mod assisted_inject;

#[derive(Default)]
pub struct InjectorInner {
  injected_instances: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
  loading_injected_instances: HashMap<TypeId, String>,
}

#[derive(Default, Clone, Component, Injectable)]
pub struct Injector {
  inner: Arc<RwLock<InjectorInner>>,
}

#[actor]
impl Injector {
  pub async fn get<C>(&self) -> C::HandleWrapper where
    C: Component + Sync,
    C::HandleWrapper: InjectableInstance<Inner=C>
  {
    let type_id = TypeId::of::<C::HandleWrapper>();

    let inner_guard = self.inner.upgradable_read().await;
    match inner_guard.injected_instances.get(&type_id) {
      None => {
        let mut inner_guard = RwLockUpgradableReadGuard::upgrade(inner_guard).await;
        if inner_guard.loading_injected_instances.insert(type_id, type_name::<C>().to_string()).is_some() {
          panic!("detected circular reference. {:?}", &inner_guard.loading_injected_instances.values());
        }
        let new_injected_instance: Lazy<<<<C as HasHandleWrapper>::HandleWrapper as InjectableInstance>::Inner as HasHandleWrapper>::HandleWrapper> = Lazy::run({
          let injector = self.clone();
          async move {
            let inner = C::HandleWrapper::create_instance(injector.clone()).await;
            let handle = inner.start();
            handle
          }
        });

        inner_guard.injected_instances.insert(type_id, Arc::new(new_injected_instance.clone()));
        drop(inner_guard);
        let new_injected_instance = new_injected_instance.get().await;
        self.inner.write().await.loading_injected_instances.remove(&type_id);
        new_injected_instance
      }

      Some(injected_instance) => {
        if self.inner.read().await.loading_injected_instances.contains_key(&type_id) {
          panic!("detected circular reference. {:?}", &inner_guard.loading_injected_instances.values());
        }
        injected_instance.clone().deref().downcast_ref::<Lazy<C::HandleWrapper>>().unwrap().get().await
      }
    }
  }
}

