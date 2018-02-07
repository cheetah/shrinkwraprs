//! # shrinkwraprs
//!
//! Making wrapper types allows us to give more compile-time
//! guarantees about our code being correct:
//!
//! ```ignore
//! // Now we can't mix up widths and heights; the compiler will yell at us!
//! struct Width(u64);
//! struct Height(u64);
//! ```
//!
//! But... they're kind of a pain to work with. If you ever need to get at
//! that wrapped `u64`, you need to constantly pattern-match back and forth
//! to wrap and unwrap the values.
//!
//! `shrinkwraprs` aims to alleviate this pain by allowing you to derive
//! implementations of various conversion traits by deriving
//! `Shrinkwrap` and `ShrinkwrapMut`.
//!
//! ## Traits implemented
//!
//! Currently, using `#[derive(Shrinkwrap)]` will derive the following traits
//! for all structs:
//!
//! * `AsRef<InnerType>`
//! * `Borrow<InnerType>`
//! * `Deref<Target=InnerType>`
//!
//! Additionally, using `#[derive(Shrinkwrap, ShrinkwrapMut)]` will additionally
//! derive the following traits:
//!
//! * `AsMut<InnerType>`
//! * `BorrowMut<InnerType>`
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
//!     let email = Email("chiya+snacks@natsumeya.jp".into());
//!
//!     let is_discriminated_email =
//!         email.contains("+");  // Woohoo, we can use the email like a string!
//!
//!     /* ... */
//! }
//! ```
//!
//! If you have multiple fields, but there's only one field you want to be able
//! to deref/borrow as, mark it with `#[shrinkwrap(main_field)]`:
//!
//! ```ignore
//! #[derive(Shrinkwrap)]
//! struct Email {
//!     spamminess: f64,
//!     #[shrinkwrap(main_field)] addr: String
//! }
//!
//! #[derive(Shrinkwrap)]
//! struct CodeSpan(u32, u32, #[shrinkwrap(main_field)] Token);
//! ```
//!
//! If you also want to be able to modify the wrapped value directly,
//! derive `ShrinkwrapMut` as well:
//!
//! ```ignore
//! #[derive(Shrinkwrap, ShrinkwrapMut)]
//! struct InputBuffer {
//!     buffer: String
//! }
//!
//! ...
//! let mut input_buffer = /* ... */;
//! input_buffer.push_str("some values");
//! ...
//! ```

// Additionally, perhaps subsume some functionality from
// [`from_variants`](https://crates.io/crates/from_variants)?

#![cfg_attr(feature = "strict", deny(warnings))]
#![recursion_limit="128"]

extern crate proc_macro;
extern crate syn;
#[macro_use] extern crate quote;
extern crate itertools;

use proc_macro::TokenStream;
use quote::{Tokens, ToTokens};

mod ast;

#[proc_macro_derive(Shrinkwrap, attributes(shrinkwrap))]
pub fn shrinkwrap(tokens: TokenStream) -> TokenStream {
  use ast::validate_derive_input;

  let input: syn::DeriveInput = syn::parse(tokens)
    .unwrap();
  let (details, input) = validate_derive_input(input);

  let mut tokens = Tokens::new();

  impl_immut_borrows(&details, &input)
    .to_tokens(&mut tokens);

  tokens.to_string()
    .parse()
    .unwrap()
}

#[proc_macro_derive(ShrinkwrapMut, attributes(shrinkwrap))]
pub fn shrinkwrap_mut(tokens: TokenStream) -> TokenStream {
  use ast::validate_derive_input;

  let input: syn::DeriveInput = syn::parse(tokens)
    .unwrap();
  let (details, input) = validate_derive_input(input);

  let mut tokens = Tokens::new();

  impl_mut_borrows(&details, &input)
    .to_tokens(&mut tokens);

  tokens.to_string()
    .parse()
    .unwrap()
}

// When generating our code, we need to be careful not to leak things into the
// surrounding code. For example, we don't use imports unless they're inside a
// scope, because otherwise we'd be inserting invisible imports whenever a user
// used #[derive(Shrinkwrap)].

fn impl_immut_borrows(details: &ast::StructDetails, input: &ast::Struct) -> Tokens {
  let &ast::StructDetails { ref ident, .. } = details;
  let &ast::Struct { ref inner_field, ref inner_type, .. } = input;

  quote! {
    impl ::std::ops::Deref for #ident {
      type Target = #inner_type;
      fn deref(&self) -> &Self::Target {
        &self.#inner_field
      }
    }

    impl ::std::borrow::Borrow<#inner_type> for #ident {
      fn borrow(&self) -> &#inner_type {
        &self.#inner_field
      }
    }

    impl ::std::convert::AsRef<#inner_type> for #ident {
      fn as_ref(&self) -> &#inner_type {
        &self.#inner_field
      }
    }
  }
}

fn impl_mut_borrows(details: &ast::StructDetails, input: &ast::Struct) -> Tokens {
  let &ast::StructDetails { ref ident, .. } = details;
  let &ast::Struct { ref inner_field, ref inner_type, .. } = input;

  quote! {
    impl ::std::ops::DerefMut for #ident {
      fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.#inner_field
      }
    }

    impl ::std::borrow::BorrowMut<#inner_type> for #ident {
      fn borrow_mut(&mut self) -> &mut #inner_type {
        &mut self.#inner_field
      }
    }

    impl ::std::convert::AsMut<#inner_type> for #ident {
      fn as_mut(&mut self) -> &mut #inner_type {
        &mut self.#inner_field
      }
    }
  }
}
