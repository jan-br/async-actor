use proc_macro::TokenStream;
use syn::Result;

mod actor_impl;
mod actor_handle_impl;
mod actor_impl_impl;
mod singleton_impl;
mod inject_impl;

#[proc_macro_attribute]
pub fn actor(args: TokenStream, input: TokenStream) -> TokenStream {
  actor_impl::actor(args, input)
}

#[proc_macro_attribute]
pub fn actor_impl(args: TokenStream, input: TokenStream) -> TokenStream {
  actor_impl_impl::actor_impl(args, input)
}

#[proc_macro_attribute]
pub fn actor_handle(args: TokenStream, input: TokenStream) -> TokenStream {
  actor_handle_impl::actor_handle(args, input)
}

#[proc_macro_derive(Singleton, attributes(inject))]
pub fn singleton(input: TokenStream) -> TokenStream {
  singleton_impl::singleton(input)
}

#[proc_macro_attribute]
pub fn inject(args: TokenStream, input: TokenStream) -> TokenStream {
  inject_impl::inect(args, input)
}