#![recursion_limit="128"]

extern crate proc_macro;
extern crate syn;
#[macro_use] extern crate quote;

use proc_macro::TokenStream;

mod ast;

#[proc_macro_derive(Shrinkwrap, attributes(shrinkwrap))]
pub fn shrinkwrap(tokens: TokenStream) -> TokenStream {
  use ast::{validate_derive_input, ShrinkwrapInput};

  let input: syn::DeriveInput = syn::parse(tokens)
    .unwrap();
  let input = validate_derive_input(input);

  let tokens = match input {
    ShrinkwrapInput::Tuple(tuple) => impl_tuple_struct(tuple),
    ShrinkwrapInput::Single(single) => impl_single_field_struct(single),
    ShrinkwrapInput::Multi(multi) => impl_multi_field_struct(multi)
  };

  tokens.to_string()
    .parse()
    .unwrap()
}

fn impl_tuple_struct(input: ast::TupleStruct) -> quote::Tokens {
  unimplemented!()
}

fn impl_single_field_struct(input: ast::SingleFieldStruct) -> quote::Tokens {
  unimplemented!()
}

fn impl_multi_field_struct(input: ast::MultiFieldStruct) -> quote::Tokens {
  unimplemented!()
}
