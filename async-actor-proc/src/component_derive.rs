use proc_macro::TokenStream;
use syn::{ItemImpl, ItemStruct, Result};
use quote::{quote};
use proc_macro2::{TokenStream as TokenStream2};
use crate::util::{format_generic_constraints, format_generic_definition, format_handle_name, format_name};

pub fn component_derive(input: TokenStream) -> TokenStream {
  let input = TokenStream2::from(input);
  let result = match parse_and_expand(input.clone()) {
    Ok(token_stream) => token_stream,
    Err(parse_error) => parse_error.to_compile_error(),
  };
  TokenStream::from(result)
}

fn parse_and_expand(input: TokenStream2) -> Result<TokenStream2> {
  let item = parse(input.clone())?;
  expand(&item)
}


fn parse(input: TokenStream2) -> Result<ItemStruct> {
  Ok(syn::parse2(input)?)
}

fn expand(mut original: &ItemStruct) -> Result<TokenStream2> {
  let component_impl = create_component_impl(&original)?;
  let component_handle = create_component_handle(&original)?;
  Ok(quote! {
    #component_handle
    #(#component_impl)*
  })
}

fn create_component_handle(original: &ItemStruct) -> Result<ItemStruct> {
  let original_name = format_name(&original.ident);
  let handle_name = format_handle_name(&original.ident);
  let generic_definition = format_generic_definition(&original.generics);
  let generic_constraints = format_generic_constraints(&original.generics);

  syn::parse2(quote! {
    #[derive(Clone)]
    pub struct #handle_name #generic_definition #generic_constraints {
      inner: async_actor::system::ComponentHandle<#original_name #generic_definition>,
    }
  })
}

fn create_component_impl(original: &ItemStruct) -> Result<Vec<ItemImpl>> {
  let original_name = format_name(&original.ident);
  let handle_name = format_handle_name(&original.ident);
  let generic_definition = format_generic_definition(&original.generics);
  let generic_constraints = format_generic_constraints(&original.generics);

  Ok(vec![
    syn::parse2(quote! {
      impl #generic_definition  async_actor::system::Component for #original_name  #generic_definition #generic_constraints {
        fn create_wrapper(inner: async_actor::system::ComponentHandle<Self>) -> Self::HandleWrapper {
          #handle_name { inner }
        }
      }
    })?,
    syn::parse2(quote!{
      impl #generic_definition async_actor::system::HasHandleWrapper for #original_name #generic_definition #generic_constraints {
        type HandleWrapper = #handle_name #generic_definition;
      }
    })?
  ])
}