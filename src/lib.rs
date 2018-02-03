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
//! For single-field structs, the following traits and methods are derived:
//!
//! * `AsRef<InnerType>`
//! * `AsMut<InnerType>`
//! * `Borrow<InnerType>`
//! * `BorrowMut<InnerType>`
//! * `Deref<Target=InnerType>`
//! * `DerefMut<Target=InnerType>`
//! * `From<InnerType> for YourType`
//! * `From<YourType> for InnerType`
//! * a `new()` constructor for `YourType` with the same visibility as the type
//!
//! For multi-field structs, all of these are derived except for
//! `From<InnerType> for YourType` and the `new()` constructor.

#![recursion_limit="128"]

extern crate proc_macro;
extern crate proc_macro2;
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

// Note that when implementing tuple structs, we don't actually care about the
// visibility of the struct itself, since we know that the tuple's inner field
// will always have the same visibility as the struct.

fn impl_tuple(input: ast::Tuple) -> quote::Tokens {
  let ast::Tuple { details, inner_type } = input;
  let ast::StructDetails { ident, visibility } = details;

  // We wrap the output in a constant to avoid leaking imports into the
  // surrounding code.
  let dummy_const = syn::Ident::new(
    &format!("__IMPL_SHRINKWRAP_FOR_{}", ident),
    proc_macro2::Span::def_site()
  );

  quote! {
    #[allow(non_upper_case_globals)]
    #[allow(unused_imports)]
    const #dummy_const: () = {
      use std::convert::{From, AsRef, AsMut};
      use std::borrow::{Borrow, BorrowMut};
      use std::ops::{Deref, DerefMut};

      impl #ident {
        #visibility fn new(input: #inner_type) -> Self {
          #ident(input)
        }
      }

      impl Deref for #ident {
        type Target = #inner_type;
        fn deref(&self) -> &#inner_type {
          &self.0
        }
      }

      impl DerefMut for #ident {
        fn deref_mut(&mut self) -> &mut #inner_type {
          &mut self.0
        }
      }

      impl Borrow<#inner_type> for #ident {
        fn borrow(&self) -> &#inner_type {
          &self.0
        }
      }

      impl BorrowMut<#inner_type> for #ident {
        fn borrow_mut(&mut self) -> &mut #inner_type {
          &mut self.0
        }
      }

      impl AsRef<#inner_type> for #ident {
        fn as_ref(&self) -> &#inner_type {
          &self.0
        }
      }

      impl AsMut<#inner_type> for #ident {
        fn as_mut(&mut self) -> &mut #inner_type {
          &mut self.0
        }
      }

      impl From<#inner_type> for #ident {
        fn from(input: #inner_type) -> Self {
          #ident(input)
        }
      }

      impl From<#ident> for #inner_type {
        fn from(input: #ident) -> #inner_type {
          input.0
        }
      }
    };
  }
}

fn impl_nary_tuple(input: ast::NaryTuple) -> quote::Tokens {
  unimplemented!()
}

// For now, we don't care about introspecting on the field visibility to figure
// out potential correctness violations.

fn impl_single(input: ast::Single) -> quote::Tokens {
  use syn::Visibility::{Public, Crate, Restricted};

  let ast::Single { details, inner_field, inner_type, inner_visibility } = input;
  let ast::StructDetails { ident, visibility } = details;

  quote! {

  }
}

fn impl_multi(input: ast::Multi) -> quote::Tokens {
  unimplemented!()
}
