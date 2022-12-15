use proc_macro::TokenStream;
use convert_case::Case::UpperCamel;
use quote::{format_ident, quote};
use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::{ItemFn, ItemStruct, Result};
use syn::parse::Nothing;

pub fn actor_handle(args: TokenStream, input: TokenStream) -> TokenStream {
  input
}
