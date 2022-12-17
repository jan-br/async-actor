use proc_macro::TokenStream;

mod singleton_derive;
mod inject_impl;
mod component_derive;
mod util;
mod actor_proc;
mod assisted_factory_proc;
mod assisted_instantiable_derive;

#[proc_macro_attribute]
pub fn actor(args: TokenStream, input: TokenStream) -> TokenStream {
  actor_proc::actor_proc(args, input)
}

#[proc_macro_derive(Singleton, attributes(inject))]
pub fn singleton(input: TokenStream) -> TokenStream {
  singleton_derive::singleton_derive(input)
}

#[proc_macro_derive(AssistedInstantiable, attributes(inject))]
pub fn assisted_instantiable(input: TokenStream) -> TokenStream {
  assisted_instantiable_derive::assisted_instantiable_derive(input)
}

#[proc_macro_derive(Component)]
pub fn component_derive(input: TokenStream) -> TokenStream {
  component_derive::component_derive(input)
}

#[proc_macro_attribute]
pub fn inject(args: TokenStream, input: TokenStream) -> TokenStream {
  inject_impl::inject(args, input)
}

#[proc_macro_attribute]
pub fn assisted_factory(args: TokenStream, input: TokenStream) -> TokenStream {
  assisted_factory_proc::assisted_factory(args, input)
}