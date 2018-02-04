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
//! Currently, `shrinkwraprs` derives the following traits for all structs:
//!
//! * `AsRef<InnerType>`
//! * `AsMut<InnerType>`
//! * `Borrow<InnerType>`
//! * `BorrowMut<InnerType>`
//! * `Deref<Target=InnerType>`
//! * `DerefMut<Target=InnerType>`
//!
//! ## Cool, how do I use it?
//!
//! ```ignore
//! #[macro_use] extern crate shrinkwraprs;
//!
//! #[derive(Shrinkwrap)]
//! struct Email(String);
//!
//! fn main() {
//!   let email = Email("chiya+snacks@natsumeya.jp".into());
//!
//!   let is_discriminated_email =
//!     (*email).contains("+");  // Woohoo, we can use the email like a string!
//!
//!   /* ... */
//! }
//! ```

// We'll probably also want to implement some other conversion traits, namely
// `From`, plus some constructors for the type itself.
//
// Additionally, perhaps subsume some functionality from
// [`from_variants`](https://crates.io/crates/from_variants)?
//
// Note: correctness concerns arise from implementing the `Mut` traits
// willy-nilly. Probably want to lock those behind visibility barriers
// for all structs.

#![cfg_attr(feature = "strict", deny(warnings))]
#![recursion_limit="128"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use] extern crate quote;
extern crate itertools;

use proc_macro::TokenStream;
use quote::{Tokens, ToTokens};

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

// When generating our code, we need to be careful not to leak anything we
// don't intend to, into the surrounding code. For example, we don't use
// imports unless they're inside a scope, because otherwise we'd be inserting
// invisible imports whenever a user used #[derive(Shrinkwrap)].

struct GenBorrowInfo {
  /// What should the `impl` keyword look like? `impl`, `impl<T>`, `impl<'a, T>`, etc.
  impl_prefix: Tokens,
  /// Should also include any generic parameters for the struct.
  struct_name: Tokens,
  inner_type: Tokens,
  /// An expression that takes in `self` and *moves* the inner field as its return value.
  borrow_expr: Tokens
}

fn impl_immut_borrows(info: &GenBorrowInfo) -> Tokens {
  let &GenBorrowInfo {
    ref impl_prefix,
    ref struct_name,
    ref inner_type,
    ref borrow_expr
  } = info;

  quote! {
    #impl_prefix ::std::ops::Deref for #struct_name {
      type Target = #inner_type;
      fn deref(&self) -> &Self::Target {
        &#borrow_expr
      }
    }

    #impl_prefix ::std::borrow::Borrow<#inner_type> for #struct_name {
      fn borrow(&self) -> &#inner_type {
        &#borrow_expr
      }
    }

    #impl_prefix ::std::convert::AsRef<#inner_type> for #struct_name {
      fn as_ref(&self) -> &#inner_type {
        &#borrow_expr
      }
    }
  }
}

// We separate out mutable borrow traits from the immutable borrows because
// later we might want to differ whether we implement mutable borrows based
// on struct visibility.

fn impl_mut_borrows(info: &GenBorrowInfo) -> Tokens {
  let &GenBorrowInfo {
    ref impl_prefix,
    ref struct_name,
    ref inner_type,
    ref borrow_expr
  } = info;

  quote! {
    #impl_prefix ::std::ops::DerefMut for #struct_name {
      fn deref_mut(&mut self) -> &mut Self::Target {
        &mut #borrow_expr
      }
    }

    #impl_prefix ::std::borrow::BorrowMut<#inner_type> for #struct_name {
      fn borrow_mut(&mut self) -> &mut #inner_type {
        &mut #borrow_expr
      }
    }

    #impl_prefix ::std::convert::AsMut<#inner_type> for #struct_name {
      fn as_mut(&mut self) -> &mut #inner_type {
        &mut #borrow_expr
      }
    }
  }
}

#[allow(unused_variables)]
fn impl_tuple(input: ast::Tuple) -> Tokens {
  let ast::Tuple { details, inner_type } = input;
  let ast::StructDetails { ident, visibility } = details;

  let gen_info = GenBorrowInfo {
    impl_prefix: quote!( impl ),
    struct_name: quote!( #ident ),
    inner_type: quote!( #inner_type ),
    borrow_expr: quote!( self.0 )
  };

  let mut tokens = Tokens::new();

  impl_immut_borrows(&gen_info)
    .to_tokens(&mut tokens);
  impl_mut_borrows(&gen_info)
    .to_tokens(&mut tokens);

  tokens
}

#[allow(unused_variables)]
fn impl_nary_tuple(input: ast::NaryTuple) -> Tokens {
  let ast::NaryTuple { details, inner_field_index, inner_type } = input;
  let ast::StructDetails { ident, visibility } = details;

  let gen_info = GenBorrowInfo {
    impl_prefix: quote!( impl ),
    struct_name: quote!( #ident ),
    inner_type: quote!( #inner_type ),
    borrow_expr: quote!( self.#inner_field_index )
  };

  let mut tokens = Tokens::new();

  impl_immut_borrows(&gen_info)
    .to_tokens(&mut tokens);
  impl_mut_borrows(&gen_info)
    .to_tokens(&mut tokens);

  tokens
}

#[allow(unused_variables)]
fn impl_single(input: ast::Single) -> Tokens {
  let ast::Single { details, inner_field, inner_type, inner_visibility } = input;
  let ast::StructDetails { ident, visibility } = details;

  let gen_info = GenBorrowInfo {
    impl_prefix: quote!( impl ),
    struct_name: quote!( #ident ),
    inner_type: quote!( #inner_type ),
    borrow_expr: quote!( self.#inner_field )
  };

  let mut tokens = Tokens::new();

  impl_immut_borrows(&gen_info)
    .to_tokens(&mut tokens);
  impl_mut_borrows(&gen_info)
    .to_tokens(&mut tokens);

  tokens
}

#[allow(unused_variables)]
fn impl_multi(input: ast::Multi) -> Tokens {
  let ast::Multi { details, inner_field, inner_type, inner_visibility } = input;
  let ast::StructDetails { ident, visibility } = details;

  let gen_info = GenBorrowInfo {
    impl_prefix: quote!( impl ),
    struct_name: quote!( #ident ),
    inner_type: quote!( #inner_type ),
    borrow_expr: quote!( self.#inner_field )
  };

  let mut tokens = Tokens::new();

  impl_immut_borrows(&gen_info)
    .to_tokens(&mut tokens);
  impl_mut_borrows(&gen_info)
    .to_tokens(&mut tokens);

  tokens
}
