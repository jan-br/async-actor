use std::any::{Any, type_name, TypeId};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use async_lock::{RwLock, RwLockUpgradableReadGuard};
use crate::inject::singleton::Singleton;
use crate::system::{Component, ComponentMessageHandler, ComponentHandle};
use async_actor_proc::{actor, actor_handle, actor_impl};
use crate::util::lazy::Lazy;

pub mod singleton;

#[derive(Default)]
pub struct InjectorInner {
  singletons: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
  loading_singletons: HashMap<TypeId, String>,
}

#[derive(Default, Clone)]
pub struct Injector {
  inner: Arc<RwLock<InjectorInner>>,
}

impl Injector {
  pub async fn get<C>(&self) -> C::HandleWrapper where
    C: Component + Sync,
    C::HandleWrapper: Singleton<Component=C>
  {
    let type_id = TypeId::of::<C::HandleWrapper>();

    let inner_guard = self.inner.upgradable_read().await;
    match inner_guard.singletons.get(&type_id) {
      None => {
        let mut inner_guard = RwLockUpgradableReadGuard::upgrade(inner_guard).await;
        if inner_guard.loading_singletons.insert(type_id, type_name::<C>().to_string()).is_some() {
          panic!("detected circular reference. {:?}", &inner_guard.loading_singletons.values());
        }
        let new_singleton: Lazy<<<<C as Component>::HandleWrapper as Singleton>::Component as Component>::HandleWrapper> = Lazy::run({
          let injector = self.clone();
          async move {
            let inner = C::HandleWrapper::create(injector.clone()).await;
            let handle = inner.start();
            handle
          }
        });

        inner_guard.singletons.insert(type_id, Arc::new(new_singleton.clone()));
        drop(inner_guard);
        let new_singleton = new_singleton.get().await;
        self.inner.write().await.loading_singletons.remove(&type_id);
        new_singleton
      }

      Some(singleton) => {
        if self.inner.read().await.loading_singletons.contains_key(&type_id) {
          panic!("detected circular reference. {:?}", &inner_guard.loading_singletons.values());
        }
        singleton.clone().deref().downcast_ref::<Lazy<C::HandleWrapper>>().unwrap().get().await
      }
    }
  }
}

