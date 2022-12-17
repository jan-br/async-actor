use proc_macro::TokenStream;
use syn::{Field, Fields, FieldsNamed, ItemImpl, ItemStruct, Result};
use quote::{format_ident, quote};
use proc_macro2::{TokenStream as TokenStream2};
use crate::util::{format_generic_constraints, format_generic_definition, format_handle_name, format_injectable_struct_instantiation, format_name};

pub fn injectable_instance_derive(input: TokenStream) -> TokenStream {
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
  let original_name = format_name(&original.ident);
  let handle_name = format_handle_name(&original.ident);
  let generic_definition = format_generic_definition(&original.generics);
  let generic_constraints = format_generic_constraints(&original.generics);
  let instantiation = format_injectable_struct_instantiation(&original, &quote!(Self::Inner), true, None);

  Ok(quote! {
    impl #generic_definition async_actor::inject::injectable_instance::InjectableInstance for #handle_name #generic_definition #generic_constraints{
      type Inner = #original_name #generic_definition;

      fn create_instance(injector: Injector) -> core::pin::Pin<Box<dyn core::future::Future<Output=Self::Inner> + core::marker::Send + core::marker::Sync>> {
        std::boxed::Box::pin(async move {
          #instantiation
        })
      }
    }
  })
}
