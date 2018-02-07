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

// We'll probably also want to implement some other conversion traits, namely
// `From`, plus some constructors for the type itself.
//
// Additionally, perhaps subsume some functionality from
// [`from_variants`](https://crates.io/crates/from_variants)?
//
// Note: correctness concerns arise from implementing the `Mut` traits
// willy-nilly. Probably want to lock those behind visibility barriers
// for all structs.
//
// Other ideas: a struct can only `Deref` to a single type, but it can
// be `Borrow`ed or `AsRef`ed as multiple types. Maybe generate multiple
// trait implementations for multiple-fielded structs? Would have to be
// careful to avoid type collisions.

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

#[proc_macro_derive(ShrinkwrapMut, attributes(shrinkwrap))]
pub fn shrinkwrap_mut(tokens: TokenStream) -> TokenStream {
  use ast::{validate_derive_input, ShrinkwrapInput};

  let input: syn::DeriveInput = syn::parse(tokens)
    .unwrap();
  let input = validate_derive_input(input);

  let tokens = match input {
    ShrinkwrapInput::Tuple(tuple) => impl_tuple_mut(tuple),
    ShrinkwrapInput::NaryTuple(nary_tuple) => impl_nary_tuple_mut(nary_tuple),
    ShrinkwrapInput::Single(single) => impl_single_mut(single),
    ShrinkwrapInput::Multi(multi) => impl_multi_mut(multi)
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

  tokens
}

#[allow(unused_variables)]
fn impl_tuple_mut(input: ast::Tuple) -> Tokens {
  let ast::Tuple { details, inner_type } = input;
  let ast::StructDetails { ident, visibility } = details;

  let gen_info = GenBorrowInfo {
    impl_prefix: quote!( impl ),
    struct_name: quote!( #ident ),
    inner_type: quote!( #inner_type ),
    borrow_expr: quote!( self.0 )
  };

  let mut tokens = Tokens::new();

  impl_mut_borrows(&gen_info)
    .to_tokens(&mut tokens);

  tokens
}

#[allow(unused_variables)]
fn impl_nary_tuple_mut(input: ast::NaryTuple) -> Tokens {
  let ast::NaryTuple { details, inner_field_index, inner_type } = input;
  let ast::StructDetails { ident, visibility } = details;

  let gen_info = GenBorrowInfo {
    impl_prefix: quote!( impl ),
    struct_name: quote!( #ident ),
    inner_type: quote!( #inner_type ),
    borrow_expr: quote!( self.#inner_field_index )
  };

  let mut tokens = Tokens::new();

  impl_mut_borrows(&gen_info)
    .to_tokens(&mut tokens);

  tokens
}

#[allow(unused_variables)]
fn impl_single_mut(input: ast::Single) -> Tokens {
  let ast::Single { details, inner_field, inner_type, inner_visibility } = input;
  let ast::StructDetails { ident, visibility } = details;

  let gen_info = GenBorrowInfo {
    impl_prefix: quote!( impl ),
    struct_name: quote!( #ident ),
    inner_type: quote!( #inner_type ),
    borrow_expr: quote!( self.#inner_field )
  };

  let mut tokens = Tokens::new();

  impl_mut_borrows(&gen_info)
    .to_tokens(&mut tokens);

  tokens
}

#[allow(unused_variables)]
fn impl_multi_mut(input: ast::Multi) -> Tokens {
  let ast::Multi { details, inner_field, inner_type, inner_visibility } = input;
  let ast::StructDetails { ident, visibility } = details;

  let gen_info = GenBorrowInfo {
    impl_prefix: quote!( impl ),
    struct_name: quote!( #ident ),
    inner_type: quote!( #inner_type ),
    borrow_expr: quote!( self.#inner_field )
  };

  let mut tokens = Tokens::new();

  impl_mut_borrows(&gen_info)
    .to_tokens(&mut tokens);

  tokens
}
