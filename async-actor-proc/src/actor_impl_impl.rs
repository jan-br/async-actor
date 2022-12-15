use proc_macro::TokenStream;
use std::borrow::Borrow;
use std::mem::replace;
use std::ops::Deref;
use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{Block, FnArg, ImplItem, ImplItemMethod, Item, ItemFn, ItemImpl, ItemMod, ItemStruct, Pat, ReturnType, Type};
use syn::parse::Nothing;
use syn::Result;

pub fn actor_impl(args: TokenStream, input: TokenStream) -> TokenStream {
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


fn parse(args: TokenStream2, input: TokenStream2) -> Result<ItemImpl> {
  let item: ItemImpl = syn::parse2(input)?;
  let _: Nothing = syn::parse2::<Nothing>(args)?;

  Ok(item)
}


fn expand(mut original: ItemImpl) -> TokenStream2 {
  let modules = create_module(&mut original);

  if let Type::Path(type_path) = &mut *original.self_ty {
    type_path.path.segments.last_mut().unwrap().ident = format_ident!("{}Handle", type_path.path.segments.last().unwrap().ident);
  }
  quote! {
    #original
    #modules
  }
}

fn create_module(original: &mut ItemImpl) -> TokenStream2 {
  if let Type::Path(type_path) = &*original.self_ty.clone() {
    let functions = modify_functions(original);
    let mod_name = format_ident!("{}", type_path.path.get_ident().unwrap().to_string().to_case(Case::Snake));
    let params = create_params(&original, functions.iter().map(|(method, _)| method).collect());
    let handlers = create_handlers(&original, functions);

    quote! {
      #(#params
      )*
      #(#handlers
      )*
    }
  } else {
    todo!()
  }
}

fn create_params(original: &ItemImpl, functions: Vec<&ImplItemMethod>) -> Vec<ItemStruct> {
  if let Type::Path(type_path) = &*original.self_ty {
    let original_name = format_ident!("{}",  type_path.path.segments.last().unwrap().ident);
    functions.iter().map(|function| {
      let function_name = function.sig.ident.clone();
      let params_name = format_ident!("{}{}Params", original_name, convert_case::Casing::to_case(&function_name.clone().to_string(), convert_case::Case::UpperCamel));
      let inputs = function.sig.inputs.iter().filter_map(|input| {
        if let FnArg::Typed(pat_type) = input {
          if let Pat::Ident(pat_ident) = pat_type.pat.deref() {
            let ident = pat_ident.ident.clone();
            let ty = pat_type.ty.clone();
            Some(quote! { #ident: #ty, })
          } else {
            None
          }
        } else {
          None
        }
      }).collect::<Vec<_>>();


      syn::parse2(quote! {
      pub struct #params_name {
        #(#inputs
        )*
      }
    }).unwrap()
    }).collect()
  } else {
    todo!()
  }
}

fn create_handlers(original: &ItemImpl, functions: Vec<(ImplItemMethod, Block)>) -> Vec<ItemImpl> {
  if let Type::Path(type_path) = &*original.self_ty {
    let original_name = format_ident!("{}",  type_path.path.segments.last().unwrap().ident);
    functions.into_iter().map(|(mut function, old_block)| {
      let return_value = match function.sig.output.clone() {
        ReturnType::Default => {
          quote! { () }
        }
        ReturnType::Type(_, t) => {
          quote!(#t)
        }
      };
      let function_name = function.sig.ident.clone();
      let params_name = format_ident!("{}{}Params", original_name, convert_case::Casing::to_case(&function_name.clone().to_string(), convert_case::Case::UpperCamel));
      let param_names = function.sig.inputs.iter().filter_map(|input| {
        if let FnArg::Typed(input) = input {
          if let Pat::Ident(ident) = input.pat.as_ref() {
            Some(ident.ident.clone())
          } else {
            None
          }
        } else {
          None
        }
      }).collect::<Vec<_>>();

      syn::parse2(quote! {
        #[async_trait::async_trait]
        impl ComponentMessageHandler<#params_name> for #type_path {
          type Answer = #return_value;

          async fn handle(&mut self, request: #params_name, wrapper: Self::HandleWrapper) -> Self::Answer {
            let #params_name { #(#param_names,)* } = request;
            #old_block
          }

        }
      }).unwrap()
    }).collect()
  } else {
    panic!("Expected a type path");
  }
}


fn modify_functions(original: &mut ItemImpl) -> Vec<(ImplItemMethod, Block)> {
  if let Type::Path(type_path) = &*original.self_ty {
    let original_name = format_ident!("{}",  type_path.path.segments.last().unwrap().ident);

    let mut functions = Vec::new();
    for item in &mut original.items {
      if let ImplItem::Method(method) = item {
        let mut found_attribute = false;
        for segment in &method.attrs.iter().flat_map(|attr| attr.path.segments.iter()).collect::<Vec<_>>() {
          if segment.ident == "actor_handle" {
            found_attribute = true;
          }
        }
        if found_attribute {
          let function_name = method.sig.ident.clone();
          let function_name_escaped = format!("{}", function_name);
          let param_names = method.sig.inputs.iter().filter_map(|input| {
            if let FnArg::Typed(input) = input {
              if let Pat::Ident(ident) = input.pat.as_ref() {
                Some(ident.ident.clone())
              } else {
                None
              }
            } else {
              None
            }
          }).collect::<Vec<_>>();
          let params_name = format_ident!("{}{}Params", original_name, convert_case::Casing::to_case(&function_name.clone().to_string(), convert_case::Case::UpperCamel));

          if let FnArg::Receiver(val) = method.sig.inputs.first_mut().expect("Self reference is required") {
            val.mutability = None
          }else{
            todo!()
          }
          let old_block = replace(&mut method.block, syn::parse2(quote! {
          {
            self.inner.dispatch(#params_name {
              #(#param_names
              ),*
            }).await
          }
        }).unwrap());
          functions.push((method.clone(), old_block));
        }
      }
    }
    functions
  } else {
    todo!()
  }
}

