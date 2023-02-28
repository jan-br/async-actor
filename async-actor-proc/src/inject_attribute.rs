use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2};
use syn::parse::{Parse, Parser, ParseStream};
use syn::{ExprClosure, MetaNameValue, Result, Token};
use syn::punctuated::Punctuated;
use syn::token::Comma;

pub fn inject(args: TokenStream, input: TokenStream) -> TokenStream {
  let args = TokenStream2::from(args);
  let input = TokenStream2::from(input);
  let result = match parse_and_expand(args.clone(), input.clone()) {
    Ok(token_stream) => token_stream,
    Err(parse_error) => parse_error.to_compile_error(),
  };
  TokenStream::from(result)
}


fn parse_and_expand(args: TokenStream2, input: TokenStream2) -> Result<TokenStream2> {
  parse(args, input.clone())?;
  Ok(input)
}


fn parse(args: TokenStream2, input: TokenStream2) -> Result<()> {

  Ok(())
}


