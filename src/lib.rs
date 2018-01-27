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
  let input = match validate_derive_input(input) {
    Ok(input) => input,
    Err(reason) => panic!(reason)
  };

  let tokens = match input {
    ShrinkwrapInput::Tuple(tuple) => impl_tuple_struct(tuple),
    ShrinkwrapInput::Single(single) => impl_single_field_struct(single),
    ShrinkwrapInput::Multi(multi) => impl_multi_field_struct(multi)
  };

  format!("{}", tokens)
    .parse()
    .unwrap()
}

fn impl_tuple_struct(input: ast::TupleStruct) -> quote::Tokens {
  let ast::TupleStruct { ident, inner_type } = input;

  quote! {
    use std::ops::{Deref, DerefMut};
    use std::borrow::{Borrow, BorrowMut};
    use std::convert::{AsRef, AsMut, From};

    impl Deref for #ident {
      type Target = #inner_type;
      fn deref(&self) -> &Self::Target {
        &self.0
      }
    }

    impl DerefMut for #ident {
      fn deref_mut(&mut self) -> &mut Self::Target {
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
      fn from(input: #ident) -> Self {
        input.0
      }
    }
  }
}

fn impl_single_field_struct(input: ast::SingleFieldStruct) -> quote::Tokens {
  unimplemented!()
}

fn impl_multi_field_struct(input: ast::MultiFieldStruct) -> quote::Tokens {
  unimplemented!()
}
