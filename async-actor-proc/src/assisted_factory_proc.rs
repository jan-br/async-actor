use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{FnArg, ImplItem, ImplItemMethod, Item, ItemFn, ItemImpl, ItemStruct, ItemTrait, Pat, Path, PathSegment, PatIdent, PatType, Result, ReturnType, TraitItem, Type, TypePath, Visibility, VisPublic};
use syn::__private::str;
use syn::parse::Nothing;
use syn::punctuated::Punctuated;
use syn::token::Colon2;
use crate::util::{find_non_injectable_fields, format_data_name, format_function_parameter_names, format_generic_constraints, format_generic_definition, format_generic_usage, format_impl_name, format_instantiation_data_name, format_name, format_return_type};

pub fn assisted_factory(args: TokenStream, input: TokenStream) -> TokenStream {
  let args = TokenStream2::from(args);
  let input = TokenStream2::from(input);
  let result = match parse_and_expand(args.clone(), input.clone()) {
    Ok(token_stream) => token_stream,
    Err(parse_error) => parse_error.to_compile_error(),
  };
  TokenStream::from(result)
}


fn parse_and_expand(args: TokenStream2, input: TokenStream2) -> Result<TokenStream2> {
  let mut item = parse(args, input)?;
  expand(&mut item)
}


fn parse(args: TokenStream2, input: TokenStream2) -> Result<ItemTrait> {
  syn::parse2::<Nothing>(args)?;
  syn::parse2(input)
}

fn expand(original: &mut ItemTrait) -> Result<TokenStream2> {
  let factory_handle = create_factory_handle(original)?;
  let factory_implementation = create_factory_implementation(original)?;

  Ok(quote! {
    #[async_trait::async_trait]
    #original
    #factory_handle
    #factory_implementation
  })
}

fn create_factory_handle(original: &mut ItemTrait) -> Result<ItemStruct> {
  let original_name = format_name(&original.ident);
  let impl_name = format_impl_name(&original.ident);
  let generic_definition = format_generic_definition(&original.generics);
  let generic_constraints = format_generic_constraints(&original.generics);

  syn::parse2(quote! {
    #[derive(Clone, async_actor_proc::Component, async_actor_proc::Injectable)]
    pub struct #impl_name #generic_definition #generic_constraints {
      #[inject] injector: async_actor::inject::InjectorHandle
    }
  })
}

fn create_factory_implementation(original: &mut ItemTrait) -> Result<ItemImpl> {
  let original_name = format_name(&original.ident);
  let impl_name = format_impl_name(&original.ident);
  let generic_definition = format_generic_definition(&original.generics);
  let generic_constraints = format_generic_constraints(&original.generics);

  let mut functions: Vec<ImplItemMethod> = vec![];
  for item in original.items.clone() {
    if let TraitItem::Method(mut function) = item {
      let return_type_name = format_return_type(&function.sig.output);
      let instantiation_data_name = format_instantiation_data_name(&format_ident!("{}", return_type_name.clone().to_string()));
      let parameter_names = format_function_parameter_names(&function.sig.inputs.iter());

      function.default = Some(syn::parse2(quote! {{
        let data = #instantiation_data_name #generic_definition {
          _phantom: core::default::Default::default(),
          #parameter_names
        };

        #return_type_name::instantiate(self.injector.clone(), data).await
      }}).unwrap());
      functions.push(ImplItemMethod {
        sig: function.sig,
        attrs: function.attrs,
        block: function.default.unwrap(),
        vis: Visibility::Inherited,
        defaultness: None,
      });
    }
  }

  syn::parse2(quote! {
    #[async_actor_proc::actor]
    #[async_trait::async_trait]
    impl #generic_definition #original_name #generic_definition for #impl_name #generic_constraints {
      #(#functions)*
    }
  })
}