use proc_macro::TokenStream;
use std::ops::Deref;

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{FnArg, ImplItem, ImplItemMethod, Item, ItemImpl, ItemStruct, Result, ReturnType};
use syn::FnArg::Receiver;
use syn::parse::Nothing;
use syn::punctuated::{Iter, Punctuated};
use syn::token::{Async, Comma};

use crate::util::{filter_function_parameters, format_data_name, format_function_parameter_definitions, format_function_parameter_names, format_generic_constraints, format_generic_definition, format_generic_usage, format_generics_as_tuple, format_handle_self_ty, format_name, format_return_type, format_self_ty, merge_generics};

pub fn actor_proc(args: TokenStream, input: TokenStream) -> TokenStream {
  let args = TokenStream2::from(args);
  let input = TokenStream2::from(input);
  let result = match parse_and_expand(args.clone(), input.clone()) {
    Ok(token_stream) => token_stream,
    Err(parse_error) => parse_error.to_compile_error(),
  };
  TokenStream::from(result)
}


fn parse_and_expand(args: TokenStream2, input: TokenStream2) -> Result<TokenStream2> {
  let item = parse(args, input)?;
  expand(&item)
}


fn parse(args: TokenStream2, input: TokenStream2) -> Result<ItemImpl> {
  syn::parse2::<Nothing>(args)?;
  syn::parse2(input)
}

fn expand(original: &ItemImpl) -> Result<TokenStream2> {
  let component_message_handler_impl = create_component_message_handler_impl(original)?;
  Ok(quote! {
    #original
    #(#component_message_handler_impl)*
  })
}

fn create_component_message_handler_impl(original: &ItemImpl) -> Result<Vec<Item>> {
  let handle_name = format_handle_self_ty(&original.self_ty);
  let generic_definitions = format_generic_definition(&original.generics);
  let generic_constraints = format_generic_constraints(&original.generics);

  let mut functions = create_wrapper_functions(original)?;
  let mut functions_data = create_wrapper_functions_data(original, &functions)?;
  let mut function_handler = create_function_handler(original, &functions)?;

  for function in &mut functions {
    function.sig.asyncness = Some(Default::default());
  }

  let mut result = vec![];


  result.push(Item::Impl(syn::parse2(quote! {
    impl #generic_definitions #handle_name #generic_constraints {
      #(#functions)*
    }
  })?));

  result.append(&mut functions_data);
  result.append(&mut function_handler);
  Ok(result)
}

fn create_function_handler(original: &ItemImpl, functions: &Vec<ImplItemMethod>) -> Result<Vec<Item>> {
  let mut result = vec![];

  let original_name = format_self_ty(&original.self_ty);

  for function in functions {
    let function_name = format_name(&function.sig.ident);
    let data_name = format_data_name(&original_name, &function.sig.ident);
    let return_name = format_return_type(&function.sig.output);
    let parameter_names = format_function_parameter_names(&function.sig.inputs.iter());
    let merged_generics = merge_generics(vec![function.sig.generics.clone(), original.generics.clone()]);
    let generic_definitions = format_generic_definition(&merged_generics);
    let generic_usage = format_generic_usage(&merged_generics);
    let generic_constraints = format_generic_constraints(&merged_generics);

    let await_maybe = &match function.sig.asyncness {
      None => quote!(),
      Some(_) => quote!(.await)
    };

    result.push(Item::Impl(syn::parse2(quote! {
      #[async_trait::async_trait]
      impl #generic_definitions async_actor::system::ComponentMessageHandler<#data_name #generic_definitions> for #original_name #generic_constraints {
        type Answer = #return_name;

        async fn handle(&mut self, request: #data_name #generic_definitions, wrapper: std::sync::Arc<Self::HandleWrapper>) -> Self::Answer {
          let #data_name #generic_usage { #parameter_names .. } = request;
          self.#function_name #generic_usage (#parameter_names)#await_maybe
        }

      }
    })?));
  }
  Ok(result)
}


fn create_wrapper_functions_data(original: &ItemImpl, functions: &Vec<ImplItemMethod>) -> Result<Vec<Item>> {
  let mut data = vec![];
  let original_name = format_self_ty(&original.self_ty.deref());
  for function in functions {
    let data_name = format_data_name(&original_name, &function.sig.ident);
    let merged_generics = merge_generics(vec![function.sig.generics.clone(), original.generics.clone()]);
    let generic_definition = format_generic_definition(&merged_generics);
    let generic_constraints = format_generic_constraints(&merged_generics);
    let generic_tuple_usage = format_generics_as_tuple(&merged_generics);
    let parameter_definitions = format_function_parameter_definitions(&function.sig.inputs.iter());
    let parameter_names = format_function_parameter_names(&function.sig.inputs.iter());
    let parameters = filter_function_parameters(&function.sig.inputs.iter());


    data.push(Item::Struct(syn::parse2(quote! {
      pub struct #data_name #generic_definition #generic_constraints {
        _phantom: core::marker::PhantomData #generic_tuple_usage,
        #(#parameters,)*
      }
    })?));

    data.push(Item::Impl(syn::parse2(quote! {
      impl #generic_definition #data_name #generic_definition #generic_constraints {
        pub fn new(#parameter_definitions) -> Self {
          Self {
            _phantom: core::default::Default::default(),
            #parameter_names
          }
        }
      }
    })?));
  }

  Ok(data)
}

fn create_wrapper_functions(original: &ItemImpl) -> Result<Vec<ImplItemMethod>> {
  let mut functions = original.items.iter()
    .filter_map(|function| if let ImplItem::Method(method) = function { Some(method) } else { None })
    .filter(|function| function.sig.inputs.first().map(|input| if let FnArg::Receiver(receiver) = input { true } else { false }).unwrap_or(false))
    .cloned()
    .collect::<Vec<_>>();

  let original_name = format_self_ty(&original.self_ty.deref());

  for function in functions.iter_mut() {
    let merged_generics = merge_generics(vec![function.sig.generics.clone(), original.generics.clone()]);
    let generic_usage = format_generic_usage(&merged_generics);
    let data_name = format_data_name(&original_name, &function.sig.ident);
    let parameter_names = format_function_parameter_names(&function.sig.inputs.iter());

    if let Some(Receiver(receiver)) = function.sig.inputs.first_mut(){
      receiver.mutability = None;
    }
    function.block = syn::parse2(quote! {{
      self.inner.dispatch(#data_name #generic_usage ::new(#parameter_names)).await
    }})?;
  }
  Ok(functions)
}