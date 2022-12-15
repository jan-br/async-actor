use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::parse::{Nothing, Result};
use syn::{parse_quote, Attribute, FnArg, Ident, ItemFn, PatType, ReturnType, ItemStruct, ItemImpl};

pub fn actor(args: TokenStream, input: TokenStream) -> TokenStream {
  let args = TokenStream2::from(args);
  let input = TokenStream2::from(input);
  let expanded = match parse(args, input.clone()) {
    Ok(item) => expand(item),
    Err(parse_error) => {
      let compile_error = parse_error.to_compile_error();
      quote!(#compile_error #input)
    }
  };
  TokenStream::from(expanded)
}

fn parse(args: TokenStream2, input: TokenStream2) -> Result<ItemStruct> {

  let item: ItemStruct = syn::parse2(input)?;
  let _: Nothing = syn::parse2::<Nothing>(args)?;

  Ok(item)
}

fn expand(mut original: ItemStruct) -> TokenStream2 {

  let handle = create_handle(&original);
  let component_impl = create_component_impl(&original);

  quote!{
    #original
    #handle
    #component_impl
  }
}


fn create_component_impl(original: &ItemStruct) -> ItemImpl {
  let original_name = original.ident.clone();
  let handle_name = format_ident!("{}Handle", original_name);

  syn::parse2(quote!{
    impl Component for #original_name {
      type HandleWrapper = #handle_name;

      fn create_wrapper(inner: ComponentHandle<Self>) -> Self::HandleWrapper {
        #handle_name { inner }
      }
    }
  }).unwrap()
}

fn create_handle(original: &ItemStruct) -> ItemStruct {
  let original_name = original.ident.clone();
  let handle_name = format_ident!("{}Handle", original.ident);

  syn::parse2(quote!{
    #[derive(Clone)]
    pub struct #handle_name {
      inner: ComponentHandle<#original_name>,
    }
  }).unwrap()
}