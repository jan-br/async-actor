use proc_macro::TokenStream;
use syn::{ItemImpl, ItemStruct, Result};
use quote::{format_ident, quote};
use proc_macro2::{TokenStream as TokenStream2};
use crate::util::{find_non_injectable_fields, format_injectable_struct_instantiation, format_generic_constraints, format_generic_definition, format_generics_as_tuple, format_handle_name, format_instantiation_data_name, format_name};

pub fn assisted_instantiable_derive(input: TokenStream) -> TokenStream {
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
  let instantiation_params = create_instantiation_params(original)?;
  let assisted_instantiable_implementation = create_assisted_instantiable_implementation(original)?;
  Ok(quote! {
    #assisted_instantiable_implementation
    #instantiation_params
  })
}

fn create_instantiation_params(original: &ItemStruct) -> Result<ItemStruct> {
  let instantiation_data_name = format_instantiation_data_name(&original.ident);
  let non_injectable_fields = find_non_injectable_fields(original);
  let generic_definition = format_generic_definition(&original.generics);
  let generic_constraints = format_generic_constraints(&original.generics);
  let generic_tuple_usage = format_generics_as_tuple(&original.generics);

  syn::parse2(quote! {
      pub struct #instantiation_data_name #generic_definition #generic_constraints {
        _phantom: core::marker::PhantomData #generic_tuple_usage,
        #(#non_injectable_fields)*
      }
   })
}

fn create_assisted_instantiable_implementation(original: &ItemStruct) -> Result<ItemImpl> {
  let original_name = format_name(&original.ident);
  let instantiation_data_name = format_instantiation_data_name(&original.ident);
  let generic_definition = format_generic_definition(&original.generics);
  let generic_constraints = format_generic_constraints(&original.generics);
  let non_injectable_fields: Vec<_> = find_non_injectable_fields(&original).iter().map(|field| {
    let name = field.ident.clone().unwrap();
    quote!(#name)
  }).collect();
  let instantiation = format_injectable_struct_instantiation(&original, &quote! {Self}, false, Some(non_injectable_fields.clone()));

  syn::parse2(quote! {
    #[async_trait::async_trait]
    impl async_actor::inject::assisted_inject::AssistedInstantiable<#instantiation_data_name #generic_definition> for #original_name #generic_constraints {
      async fn instantiate(injector: async_actor::inject::InjectorHandle, data: #instantiation_data_name #generic_definition) -> #original_name{
        let #instantiation_data_name #generic_definition { #(#non_injectable_fields,)* .. } = data;

        #instantiation
      }
    }
  })
}
