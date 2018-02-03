//! We want to make sure that the struct that our caller passes us is in the
//! right form. However, we don't want to clutter up our code generation
//! logic with lots of error handling. So instead, we take in our `DeriveInput`
//! and do all the error handling in one place, transforming it into an AST
//! specific to our crate if it's valid.

use syn;
use quote::ToTokens;

use itertools::Itertools;

use std::borrow::Cow;

type Fields = Vec<syn::Field>;

pub struct StructDetails {
  pub ident: syn::Ident,
  pub visibility: syn::Visibility
}

/// Represents a 1-tuple struct.
pub struct Tuple {
  pub details: StructDetails,
  pub inner_type: syn::Type
}

/// Represents an n-tuple struct, with one of the elements designated
/// as the one we should deref to.
pub struct NaryTuple {
  pub details: StructDetails,
  pub inner_field_index: u32,
  pub inner_type: syn::Type
}

/// Represents a normal struct with a single named field.
pub struct Single {
  pub details: StructDetails,
  pub inner_field: syn::Ident,
  pub inner_type: syn::Type,
  pub inner_visibility: syn::Visibility
}

/// Represents a normal struct with multiple named fields, one of which we
/// should deref to.
pub struct Multi {
  pub details: StructDetails,
  pub inner_field: syn::Ident,
  pub inner_type: syn::Type,
  pub inner_visibility: syn::Visibility
}

pub enum ShrinkwrapInput {
  Tuple(Tuple),
  NaryTuple(NaryTuple),
  Single(Single),
  Multi(Multi)
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
      let fields = fields.into_iter().collect_vec();
      validate_tuple(details, fields)
    },
    Struct(DataStruct { fields: Named(FieldsNamed { named: fields, .. }), .. }) => {
      let fields = fields.into_iter().collect_vec();
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

  let mut fields = fields;
  if let Some(syn::Field { ty, .. }) = fields.pop() {
    ShrinkwrapInput::Tuple(Tuple {
      details: details,
      inner_type: ty
    })
  } else {
    unreachable!()
  }
}

fn validate_struct(details: StructDetails, fields: Vec<syn::Field>) -> ShrinkwrapInput {
  if fields.len() == 0 {
    panic!("shrinkwraprs requires structs to have at least one field");
  } else if fields.len() > 1 {
    panic!("currently, shrinkwraprs does not support structs with more than one field");
  } else {
    let mut fields = fields;
    if let Some(syn::Field { vis, ty, ident: Some(ident), .. }) = fields.pop() {
      ShrinkwrapInput::Single(Single {
        details: details,
        inner_field: ident,
        inner_type: ty,
        inner_visibility: vis
      })
    } else {
      unreachable!()
    }
  }
}
