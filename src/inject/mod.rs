use std::any::{Any, type_name, TypeId};
use std::collections::{HashMap};
use std::future::Future;
use std::marker::Unsize;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use async_lock::{RwLock, RwLockUpgradableReadGuard};
use async_actor_proc::{actor, Component, Injectable};
use crate as async_actor;
use crate::inject::injectable_instance::InjectableInstance;
use crate::system::{Component, HasHandleWrapper};
use crate::util::lazy_cell::LazyCell;

pub mod injectable_instance;
pub mod assisted_inject;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum Binding {
  Unnamed(TypeId),
  Named(TypeId, String),
}

#[derive(Default)]
pub struct InjectorInner {
  injected_instances: HashMap<Binding, Arc<dyn Any + Send + Sync>>,
  loading_injected_instances: HashMap<Binding, String>,
  mappings: HashMap<TypeId, TypeId>,
}

#[derive(Default, Clone, Component)]
pub struct Injector {
  inner: Arc<RwLock<InjectorInner>>,
}

impl InjectableInstance for InjectorHandle {
  type Inner = Injector;

  fn create_instance(injector: Injector) -> Pin<Box<dyn Future<Output=Box<Self>> + Send + Sync>> {
    Box::pin(async move {
      Box::new(injector.start())
    })
  }
}

#[actor]
impl Injector {
  pub async fn get<C>(&self) -> C::HandleWrapper
    where
      C: HasHandleWrapper + ?Sized + Send + Sync + 'static,
      C::HandleWrapper: InjectableInstance<Inner=C>,
  {
    self.get_internal::<C>(Binding::Unnamed(TypeId::of::<C>())).await
  }

  pub async fn get_outer<C>(&self) -> C
    where
      C: InjectableInstance,
      C::Inner: HasHandleWrapper<HandleWrapper=C> + Send + Sync + 'static
  {
    self.get_internal::<C::Inner>(Binding::Unnamed(TypeId::of::<C::Inner>())).await
  }


  pub async fn get_named<C>(&self, name: String) -> C::HandleWrapper
    where
      C: HasHandleWrapper + ?Sized + Send + Sync + 'static,
      C::HandleWrapper: InjectableInstance<Inner=C>,

  {
    self.get_internal::<C>(Binding::Named(TypeId::of::<C>(), name)).await
  }

  pub async fn get_outer_named<C>(&self, name: String) -> C
    where
      C: InjectableInstance<Inner=C>,
      C::Inner: HasHandleWrapper<HandleWrapper=C> + Send + Sync + 'static
  {
    self.get_internal::<C>(Binding::Named(TypeId::of::<C::Inner>(), name)).await
  }
}

impl Injector {
  pub async fn bind<T, I>(&self)
    where
      T: ?Sized + Send + 'static,
      I: Unsize<T> + HasHandleWrapper + 'static,
  {
    self.inner.write().await.mappings.insert(TypeId::of::<T>(), TypeId::of::<I>());
  }

  pub async fn bind_value<'a, T>(&'a self, value: T) -> Pin<Box<dyn Future<Output=()> + Send + Sync + 'a>> where T: Send + Sync + 'static {
    Box::pin(async move {
      let mut inner = self.inner.write().await;
      inner.injected_instances.insert(Binding::Unnamed(TypeId::of::<T>()), Arc::new(value));
    })
  }

  fn get_internal<'a, C>(&'a self, binding: Binding) -> Pin<Box<dyn Future<Output=C::HandleWrapper> + Send + Sync + 'a>>
    where
      C: HasHandleWrapper + ?Sized + Send + Sync + 'static,
      C::HandleWrapper: InjectableInstance<Inner=C>,
  {
    Box::pin(async move {
      let inner_guard = self.inner.upgradable_read().await;

      let type_id = match binding.clone() {
        Binding::Unnamed(type_id) => type_id,
        Binding::Named(type_id, _) => type_id
      };

      if let Some(new_type_id) = inner_guard.mappings.get(&type_id) {
        let mut binding = binding.clone();
        match &mut binding {
          Binding::Unnamed(old_type_id) => *old_type_id = *new_type_id,
          Binding::Named(old_type_id, _) => *old_type_id = *new_type_id
        }
        return self.get_internal::<C>(binding).await;
      }

      match inner_guard.injected_instances.get(&binding) {
        None => {
          let mut inner_guard = RwLockUpgradableReadGuard::upgrade(inner_guard).await;
          if inner_guard.loading_injected_instances.insert(binding.clone(), type_name::<C>().to_string()).is_some() {
            panic!("detected circular reference. {:?}", &inner_guard.loading_injected_instances.values());
          }
          let new_injected_instance: Arc<LazyCell<C::HandleWrapper>> = Arc::new(LazyCell::new({
            let injector = self.clone();
            async move {
              C::HandleWrapper::create_instance(injector.clone()).await.as_ref().clone()
            }
          }));

          inner_guard.injected_instances.insert(binding.clone(), new_injected_instance.clone());
          drop(inner_guard);
          let new_injected_instance = new_injected_instance.get().await.clone();
          self.inner.write().await.loading_injected_instances.remove(&binding);
          new_injected_instance
        }

        Some(injected_instance) => {
          if self.inner.read().await.loading_injected_instances.contains_key(&binding) {
            panic!("detected circular reference. {:?}", &inner_guard.loading_injected_instances.values());
          }
          injected_instance.clone().deref().downcast_ref::<LazyCell<C::HandleWrapper>>().unwrap().get().await.clone()
        }
      }
    })
  }
}
