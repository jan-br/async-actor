use async_trait::async_trait;
use crate::inject::InjectorHandle;

#[async_trait]
pub trait AssistedInstantiable<P> {
  async fn instantiate(injector: InjectorHandle, params: P) -> Self;
}