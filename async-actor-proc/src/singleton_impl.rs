use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{ItemImpl, ItemStruct, Result};
use syn::parse::Nothing;

pub fn singleton( input: TokenStream) -> TokenStream {
  let input = TokenStream2::from(input);
  let expanded = match parse(input.clone()) {
    Ok(item) => expand(item),
    Err(parse_error) => {
      let compile_error = parse_error.to_compile_error();
      quote!(#compile_error #input)
    }
  };
  TokenStream::from(expanded)
}

fn parse(input: TokenStream2) -> Result<ItemStruct> {
  let item: ItemStruct = syn::parse2(input)?;
  Ok(item)
}

fn expand(mut original: ItemStruct) -> TokenStream2 {
  let singleton = create_singleton(&original);

  quote! {
    #singleton
  }
}

fn create_singleton(original: &ItemStruct) -> ItemImpl{
  let generics = &original.generics;
  let generic_params = &generics.params.iter().collect::<Vec<_>>();
  let ident = &original.ident;
  let where_clause = &original.generics.where_clause;
  let generics_with_separator = if generic_params.is_empty() {
    quote!()
  } else {
    quote!(:: <#(#generic_params,)*>)
  };

  let fields = &original.fields.iter().map(|field| {
    let ident = &field.ident;
    let ty = &field.ty;
    let mut found_attribute = false;
    for segment in &field.attrs.iter().flat_map(|attr| attr.path.segments.iter()).collect::<Vec<_>>() {
      if segment.ident == "inject" {
        found_attribute = true;
      }
    }
    if found_attribute {
      quote! {
        #ident: injector.get::<<#ty as Singleton>::Component>().await,
      }
    } else {
      quote! {
        #ident: Default::default()
      }
    }
  }).collect::<Vec<_>>();

  syn::parse2(quote!{
    #[async_actor_proc::actor_impl]
    impl #generics Singleton for #ident #generics #where_clause {
      type Component = #ident;
      fn create(injector: Injector) -> Pin<Box<dyn Future<Output=#ident> + Send + Sync>> {
        Box::pin(async move {
          #ident #generics_with_separator {
           #(#fields)*
          }
        })
      }
    }
  }).unwrap()
}