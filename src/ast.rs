//! We want to make sure that the struct that our caller passes us is in the
//! right form. However, we don't want to clutter up our code generation
//! logic with lots of error handling. So instead, we take in our `DeriveInput`
//! and do all the error handling in one place, transforming it into an AST
//! specific to our crate if it's valid.

use syn;
use quote::ToTokens;

use std::borrow::Cow;

pub struct StructDetails {
  pub ident: syn::Ident,
  pub visibility: syn::Visibility
}

/// Represents a 1-tuple struct.
pub struct TupleStruct {
  pub details: StructDetails,
  pub inner_type: syn::Type
}

/// Represents a normal struct with a single named field.
pub struct SingleFieldStruct {
  pub details: StructDetails,
  pub inner_field: syn::Ident,
  pub inner_type: syn::Type,
  pub inner_visibility: syn::Visibility
}

/// Represents a normal struct with multiple named fields, one of which we
/// should deref to.
pub struct MultiFieldStruct {
  pub details: StructDetails,
  pub inner_field: syn::Ident,
  pub inner_type: syn::Type,
  pub inner_visibility: syn::Visibility
}

pub enum ShrinkwrapInput {
  Tuple(TupleStruct),
  Single(SingleFieldStruct),
  Multi(MultiFieldStruct)
}

pub fn validate_derive_input(input: syn::DeriveInput) -> ShrinkwrapInput {
  // Note that `unwrap()`s and `panic()`s are totally fine here, since we're
  // inside a procedural macro; panics happen at compile time

  use syn::{DeriveInput, DataStruct, FieldsUnnamed, FieldsNamed, Field};
  use syn::Data::{Struct, Enum, Union};
  use syn::Fields::{Named, Unnamed, Unit};

  let DeriveInput { attrs: _attrs, vis, ident, generics, data, .. } = input;

  if !generics.params.is_empty() {
    panic!("currently, shrinkwraprs does not support structs with lifetimes or generics");
  }

  let details = StructDetails { ident: ident, visibility: vis };

  match data {
    Struct(DataStruct { fields: Unnamed(FieldsUnnamed { unnamed: fields, .. }), .. }) => {
      let fields: Vec<Field> = fields.into_iter().collect();
      validate_tuple(details, fields)
    },
    Struct(DataStruct { fields: Named(FieldsNamed { named: fields, .. }), .. }) => {
      let fields: Vec<Field> = fields.into_iter().collect();
      validate_struct(details, fields)
    },
    Struct(..) =>
      panic!("shrinkwraprs needs a struct with at least one field!"),
    Enum(..) =>
      panic!("shrinkwraprs does not support enums"),
    Union(..) =>
      panic!("shrinkwraprs does not support C-style unions")
  }
}

fn validate_tuple(details: StructDetails, fields: Vec<syn::Field>) -> ShrinkwrapInput {
  if fields.len() == 0 {
    panic!("shrinkwraprs requires tuple structs to have at least one field");
  } else if fields.len() > 1 {
    panic!("currently, shrinkwraprs does not support tuple structs with more than one field");
  }

  unimplemented!()
}

fn validate_struct(details: StructDetails, fields: Vec<syn::Field>) -> ShrinkwrapInput {
  if fields.len() == 0 {
    panic!("shrinkwraprs requires structs to have at least one field");
  } else if fields.len() > 1 {
    panic!("currently, shrinkwraprs does not support structs with more than one field");
  }

  unimplemented!()
}
