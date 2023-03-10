use std::collections::HashMap;
use std::ops::Deref;
use std::os::linux::raw::stat;
use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenTree};
use quote::{format_ident, quote, TokenStreamExt, ToTokens};
use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::{Expr, ExprClosure, Field, FnArg, GenericParam, Generics, ItemStruct, Pat, PatType, ReturnType, Stmt, Token, Type, WhereClause, WherePredicate};
use syn::parse::Parser;
use syn::punctuated::{Iter, Punctuated};
use syn::token::Comma;

pub fn format_name(ident: &Ident) -> TokenStream2 {
    quote!(#ident)
}

pub fn convert_return_type_to_ident(rt: &ReturnType) -> Ident {
    // Convert the ReturnType into a TokenStream
    let tokens = format_return_type(rt);

    // Extract the first Ident from the TokenStream
    let mut tokens = tokens.into_iter();
    let first_token = tokens.next().unwrap();

    if let proc_macro2::TokenTree::Ident(ident) = first_token {
        // Return the Ident
        ident
    } else {
        todo!()
    }
}

fn convert_type_to_ident(ty: &Type) -> Ident {
    // Convert the Type into a TokenStream
    let tokens = quote! { #ty };

    // Extract the first Ident from the TokenStream
    let mut tokens = tokens.into_iter();
    let first_token = tokens.next().unwrap();
    if let proc_macro2::TokenTree::Ident(ident) = first_token {
        // Return the Ident
        ident
    } else {
        todo!()
    }
}

pub fn format_handle_name(ident: &Ident) -> TokenStream2 {
    format_name(&format_ident!("{}Handle", ident))
}

pub fn format_handle_name_unique(ident: &Ident) -> TokenStream2 {
    format_name(&format_ident!("{}HandleUnique", ident))
}

pub fn format_impl_name(ident: &Ident) -> TokenStream2 {
    format_name(&format_ident!("{}Impl", ident))
}

pub fn format_name_with_case(ident: &Ident, case: Case) -> TokenStream2 {
    format_name(&format_ident!("{}", quote!(#ident).to_string().to_case(case)))
}

pub fn format_data_name(prefix: &TokenStream2, ident: &Ident) -> TokenStream2 {
    format_name_with_case(&format_ident!("{}_{}_data", prefix.to_string(), ident), Case::UpperCamel)
}

pub fn format_instantiation_data_name(ident: &Ident) -> TokenStream2 {
    format_name_with_case(&format_ident!("{}_instantiation_data", ident), Case::UpperCamel)
}

pub fn filter_function_parameters(inputs: &Iter<FnArg>) -> Vec<PatType> {
    inputs.clone().filter_map(|input| if let FnArg::Typed(arg) = input { Some(arg.clone()) } else { None }).collect::<Vec<_>>()
}

pub fn format_function_parameter_definitions(inputs: &Iter<FnArg>) -> TokenStream2 {
    let parameters = filter_function_parameters(inputs);
    quote!(#(#parameters,)*)
}

pub fn format_function_parameter_names(inputs: &Iter<FnArg>) -> TokenStream2 {
    let parameters = filter_function_parameters(inputs).iter().map(|ty| match ty.pat.deref() {
        Pat::Ident(ident) => ident.ident.clone(),
        _ => unimplemented!("Please open an issue, if another case implementation is needed.")
    }).collect::<Vec<_>>();
    quote!(#(#parameters,)*)
}


pub fn format_self_ty(ty: &Type) -> TokenStream2 {
    format_name(&convert_type_to_ident(ty))
}

pub fn format_handle_self_ty(ty: &Type) -> TokenStream2 {
    let mut ty = ty.clone();
    match &mut ty {
        Type::Path(path) => {
            let ident = &mut path.path.segments.first_mut().expect("No first path segment is present").ident;
            *ident = format_ident!("{}Handle", ident.to_string());
        }
        _ => unimplemented!("Please open an issue, if another case implementation is needed.")
    }
    format_self_ty(&ty)
}

pub fn format_generic_definition(generics: &Generics) -> TokenStream2 {
    quote! {
    #generics
  }
}

pub fn merge_generics(generics: impl IntoIterator<Item=Generics>) -> Generics {
    let mut result = Generics::default();
    for generics in generics.into_iter().collect::<Vec<Generics>>() {
        for param in &generics.params {
            result.params.push(param.clone());
        }
        if let Some(where_clause) = &generics.where_clause {
            if result.where_clause.is_none() {
                result.where_clause = Some(WhereClause {
                    where_token: Default::default(),
                    predicates: Default::default(),
                });
            }
            for predicate in where_clause.predicates.iter() {
                result.where_clause.as_mut().unwrap().predicates.push(predicate.clone());
            }
        }
    }
    result
}

pub fn merge_generic_constraints(constraints: Vec<Vec<WherePredicate>>) -> WhereClause {
    let mut result = WhereClause {
        where_token: Default::default(),
        predicates: Default::default(),
    };


    for predicate in constraints.into_iter().flat_map(|vec| vec) {
        result.predicates.push(predicate);
    }
    result
}

pub fn format_generic_usage(generics: &Generics) -> TokenStream2 {
    if generics.params.is_empty() {
        quote!()
    } else {
        quote! {
      ::#generics
    }
    }
}

pub fn format_generic_usage_as_tuple(generics: &Generics) -> TokenStream2 {
    let params = &generics.params.iter().collect::<Vec<_>>();
    if generics.params.is_empty() {
        quote!(::<()>)
    } else {
        quote! {
      ::<(#(#params,)*)>
    }
    }
}

pub fn format_return_type(return_type: &ReturnType) -> TokenStream2 {
    match return_type {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, ty) => quote!(#ty)
    }
}

pub fn find_and_classify_fields(original: &ItemStruct, attributes: Vec<String>) -> Vec<(Field, bool)> {
    original.fields.clone().into_iter().map(|field| {
        let mut found_attribute = vec![];
        for attribute in &attributes {
            for segment in &field.attrs.iter().flat_map(|attr| attr.path.segments.iter()).collect::<Vec<_>>() {
                if segment.ident == attribute.clone() {
                    found_attribute.push(attribute.clone());
                }
            }
        }
        (field, found_attribute == attributes)
    }).collect::<Vec<_>>()
}

pub fn find_and_classify_injectable_fields(original: &ItemStruct) -> Vec<(Field, bool)> {
    find_and_classify_fields(original, vec!["inject".to_string()])
}

pub fn find_and_classify_default_fields(original: &ItemStruct) -> Vec<(Field, bool)> {
    find_and_classify_fields(original, vec!["inject_default".to_string()])
}

pub fn find_injectable_fields(original: &ItemStruct) -> Vec<Field> {
    let classified = find_and_classify_injectable_fields(original);
    classified.into_iter().filter(|(_, injectable)| *injectable).map(|(field, _)| field.clone()).collect::<Vec<_>>()
}

pub fn find_default_fields(original: &ItemStruct) -> Vec<Field> {
    let classified = find_and_classify_default_fields(original);
    classified.into_iter().filter(|(_, injectable)| *injectable).map(|(field, _)| field.clone()).collect::<Vec<_>>()
}

pub fn find_non_injectable_non_default_fields(original: &ItemStruct) -> Vec<Field> {
    original.fields.iter()
        .filter(|field| {
            !find_injectable_fields(original).iter().any(|other_field| other_field.ident == field.ident)
                && !find_default_fields(original).iter().any(|other_field| other_field.ident == field.ident)
        })
        .cloned()
        .collect()
}

pub fn format_injectable_struct_instantiation(original: &ItemStruct, ty: &TokenStream2, defaults_allowed: bool, extra_fields: Option<Vec<TokenStream2>>) -> TokenStream2 {
    let injectable_fields = find_injectable_fields(original);
    let default_fields = find_default_fields(original);

    let field_inject_initialization = format_field_initialization(injectable_fields, |field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        let attribute = field.attrs.iter().find_map(|attr| if attr.path.segments.last().map(|segment| segment.ident == "inject").unwrap_or(false) {
            Some(attr)
        } else {
            None
        }).unwrap();

        let initializer = if let Some(Ok(closure)) = attribute.clone().tokens.into_iter().collect::<Vec<_>>()
            .into_iter()
            .filter_map(|tree| if let TokenTree::Group(group) = tree {
                Some(syn::parse2::<ExprClosure>(group.stream()))
            } else {
                None
            })
            .collect::<Vec<_>>()
            .first()
            .cloned() {
                quote! {
                    #field_name: (#closure).call((injector.clone(),)).await
                }
            } else {
                quote! {
                    #field_name: injector.get_outer::<#field_type>().await
                }
            };

        initializer
    });

    let mut field_default_initializations = if defaults_allowed {
        format_field_initialization(default_fields, |field| {
            let field_name = &field.ident;
            quote!(#field_name: core::default::Default::default())
        })
    } else {
        quote!()
    };

    let extra_fields = extra_fields.unwrap_or_default();

    quote! {
    #ty {
      #field_inject_initialization
      #field_default_initializations
      #(#extra_fields,)*
    }
  }
}


pub fn format_field_initialization(fields: impl IntoIterator<Item=Field>, callback: impl Fn(&Field) -> TokenStream2) -> TokenStream2 {
    let field_lines = fields.into_iter().map(|field| {
        callback(&field)
    }).collect::<Vec<_>>();

    quote!(#(#field_lines,)*)
}


pub fn format_generic_constraints(generics: &Generics) -> TokenStream2 {
    let where_clause = &generics.where_clause;
    quote! {
    #where_clause
  }
}

