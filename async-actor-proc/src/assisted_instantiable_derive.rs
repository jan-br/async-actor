use proc_macro::TokenStream;
use syn::{Item, ItemImpl, ItemStruct, Result};
use quote::{format_ident, quote};
use proc_macro2::{TokenStream as TokenStream2};
use crate::util::{format_injectable_struct_instantiation, format_generic_constraints, format_generic_definition, format_generics_as_tuple, format_handle_name, format_instantiation_data_name, format_name, find_non_injectable_non_default_fields, format_generic_usage};

pub fn assisted_instantiable_derive(input: TokenStream) -> TokenStream {
  let input = TokenStream2::from(input);
  let result = match parse_and_expand(input.clone()) {
    Ok(token_stream) => token_stream,
    Err(parse_error) => parse_error.to_compile_error(),
  };
  TokenStream::from(result)
}

fn parse_and_expand(input: TokenStream2) -> Result<TokenStream2> {
  let item = parse(input)?;
  expand(&item)
}


fn parse(input: TokenStream2) -> Result<ItemStruct> {
  syn::parse2(input)
}

fn expand(mut original: &ItemStruct) -> Result<TokenStream2> {
  let instantiation_params = create_instantiation_params(original)?;
  let assisted_instantiable_implementation = create_assisted_instantiable_implementation(original)?;
  Ok(quote! {
    #assisted_instantiable_implementation
    #(#instantiation_params)*
  })
}

fn create_instantiation_params(original: &ItemStruct) -> Result<Vec<Item>> {
  let instantiation_data_name = format_instantiation_data_name(&original.ident);
  let non_injectable_non_default_fields = find_non_injectable_non_default_fields(original);
  let non_injectable_non_default_field_names = non_injectable_non_default_fields.iter().map(|field|field.ident.clone()).collect::<Vec<_>>();
  let generic_definition = format_generic_definition(&original.generics);
  let generic_constraints = format_generic_constraints(&original.generics);
  let generic_tuple_usage = format_generics_as_tuple(&original.generics);


  println!("Meh fuck {}", quote!(#(#non_injectable_non_default_fields,)*));
  Ok(vec![
    Item::Struct(
      syn::parse2(quote! {
        pub struct #instantiation_data_name #generic_definition #generic_constraints {
          _phantom: core::marker::PhantomData #generic_tuple_usage,
          #(#non_injectable_non_default_fields,)*
        }
      })?
    ),
    Item::Impl(
      syn::parse2(quote! {
        impl #generic_definition #instantiation_data_name #generic_definition #generic_constraints {
          pub fn new(#(#non_injectable_non_default_fields,)*) -> Self {
            Self {
              _phantom: core::default::Default::default(),
              #(#non_injectable_non_default_field_names,)*
            }
          }
        }
      }).expect("wtf")
    ),
  ])
}

fn create_assisted_instantiable_implementation(original: &ItemStruct) -> Result<ItemImpl> {
  let original_name = format_name(&original.ident);
  let instantiation_data_name = format_instantiation_data_name(&original.ident);
  let generic_definition = format_generic_definition(&original.generics);
  let generic_usage = format_generic_usage(&original.generics);
  let generic_constraints = format_generic_constraints(&original.generics);
  let non_injectable_non_default_fields: Vec<_> = find_non_injectable_non_default_fields(&original).iter().map(|field| {
    let name = field.ident.clone().unwrap();
    quote!(#name)
  }).collect();
  let instantiation = format_injectable_struct_instantiation(&original, &quote! {Self}, true, Some(non_injectable_non_default_fields.clone()));


  let instantiation_data_destructor = if non_injectable_non_default_fields.is_empty() {
    quote!()
  }else{
    quote!(let #instantiation_data_name #generic_usage { #(#non_injectable_non_default_fields,)* .. } = data;)
  };

  let x = quote! {
    #[async_trait::async_trait]
    impl #generic_definition async_actor::inject::assisted_inject::AssistedInstantiable<#instantiation_data_name #generic_definition> for #original_name #generic_definition #generic_constraints {
      async fn instantiate(injector: async_actor::inject::InjectorHandle, data: #instantiation_data_name #generic_definition) -> Self {
        #instantiation_data_destructor

        #instantiation
      }
    }
  };
  println!("{}", x);
  syn::parse2(x)
}
