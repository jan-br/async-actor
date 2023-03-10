use proc_macro::TokenStream;

mod injectable_instance_derive;
mod inject_attribute;
mod component_derive;
mod util;
mod actor_proc;
mod assisted_factory_proc;
mod assisted_instantiable_derive;
mod inject_default_attribute;

#[proc_macro_attribute]
pub fn actor(args: TokenStream, input: TokenStream) -> TokenStream {
  actor_proc::actor_proc(args, input)
}

#[proc_macro_derive(Injectable, attributes(inject, inject_default))]
pub fn injectable_instance(input: TokenStream) -> TokenStream {
  injectable_instance_derive::injectable_instance_derive(input)
}

#[proc_macro_derive(AssistedInstantiable, attributes(inject, inject_default))]
pub fn assisted_instantiable(input: TokenStream) -> TokenStream {
  assisted_instantiable_derive::assisted_instantiable_derive(input)
}

#[proc_macro_derive(Component)]
pub fn component_derive(input: TokenStream) -> TokenStream {
  component_derive::component_derive(input)
}

#[proc_macro_attribute]
pub fn inject(args: TokenStream, input: TokenStream) -> TokenStream {
  inject_attribute::inject(args, input)
}

#[proc_macro_attribute]
pub fn default(args: TokenStream, input: TokenStream) -> TokenStream {
  inject_default_attribute::default(args, input)
}

#[proc_macro_attribute]
pub fn assisted_factory(args: TokenStream, input: TokenStream) -> TokenStream {
  assisted_factory_proc::assisted_factory(args, input)
}