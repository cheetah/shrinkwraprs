//! # shrinkwraprs
//!
//! Making wrapper types allows us to give more compile-time
//! guarantees about our code being correct:
//!
//! ```ignore
//! // Now we can't mix up widths and heights; the compiler will yell at us!
//! struct Width(i64);
//! struct Height(i64);
//! ```
//!
//! But... they're kind of a pain to work with. If you ever need to get at
//! that wrapped `i64`, you need to constantly pattern-match back and forth
//! to wrap and unwrap the values.
//!
//! `shrinkwraprs` aims to alleviate this pain by allowing you to derive
//! implementations of various conversion traits by attaching
//! `#[derive(Shrinkwrap)]`.
//!
//! ## Traits implemented
//!
//! For single-field structs, the following traits are derived:
//!
//! * `AsRef<InnerType>`
//! * `AsMut<InnerType>`
//! * `Borrow<InnerType>`
//! * `BorrowMut<InnerType>`
//! * `Deref<Target=InnerType>`
//! * `DerefMut<Target=InnerType>`
//! * `From<InnerType> for YourType`
//! * `From<YourType> for InnerType`
//!
//! For multi-field structs, all of these are derived except for
//! `From<InnerType> for YourType`.

#![recursion_limit="128"]

extern crate proc_macro;
extern crate syn;
#[macro_use] extern crate quote;
extern crate itertools;

use proc_macro::TokenStream;

mod ast;

#[proc_macro_derive(Shrinkwrap, attributes(shrinkwrap))]
pub fn shrinkwrap(tokens: TokenStream) -> TokenStream {
  use ast::{validate_derive_input, ShrinkwrapInput};

  let input: syn::DeriveInput = syn::parse(tokens)
    .unwrap();
  let input = validate_derive_input(input);

  let tokens = match input {
    ShrinkwrapInput::Tuple(tuple) => impl_tuple(tuple),
    ShrinkwrapInput::NaryTuple(nary_tuple) => impl_nary_tuple(nary_tuple),
    ShrinkwrapInput::Single(single) => impl_single(single),
    ShrinkwrapInput::Multi(multi) => impl_multi(multi)
  };

  tokens.to_string()
    .parse()
    .unwrap()
}

fn impl_tuple(input: ast::Tuple) -> quote::Tokens {
  unimplemented!()
}

fn impl_nary_tuple(input: ast::NaryTuple) -> quote::Tokens {
  unimplemented!()
}

fn impl_single(input: ast::Single) -> quote::Tokens {
  unimplemented!()
}

fn impl_multi(input: ast::Multi) -> quote::Tokens {
  unimplemented!()
}
