//! We want to make sure that the struct that our caller passes us is in the
//! right form. However, we don't want to clutter up our code generation
//! logic with lots of error handling. So instead, we take in our `DeriveInput`
//! and do all the error handling in one place, transforming it into an AST
//! specific to our crate if it's valid.

use syn;
use quote::ToTokens;

use std::borrow::Cow;

/// Represents a 1-tuple struct.
pub struct TupleStruct {
  pub ident: syn::Ident,
  pub inner_type: syn::Type
}

/// Represents a normal struct with a single named field.
pub struct SingleFieldStruct {
  ident: String,
  inner_field: String,
  inner_type: String
}

/// Represents a normal struct with multiple named fields, one of which we
/// should deref to.
pub struct MultiFieldStruct {
  ident: String,
  inner_field: String,
  inner_type: String
}

pub enum ShrinkwrapInput {
  Tuple(TupleStruct),
  Single(SingleFieldStruct),
  Multi(MultiFieldStruct)
}

pub fn validate_derive_input<'a>(input: syn::DeriveInput)
  -> Result<ShrinkwrapInput, Cow<'a, str>>
{
  let syn::DeriveInput { attrs, ident, generics, data, .. } = input;

  let generics: Vec<syn::TypeParam> = generics.params.into_iter()
    .filter_map(|generic| match generic {
      syn::GenericParam::Type(param) => Some(param),
      _ => None
    })
    .collect();

  if !generics.is_empty() {
    return Err("right now, shrinkwraprs does not support structs with generic parameters.".into());
  }

  match data {
    syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Unnamed(fields), .. }) => {
      let mut fields: Vec<syn::Field> = fields.unnamed.into_iter()
        .collect();
      if fields.len() != 1 {
        return Err("shrinkwraprs does not support tuple structs with more than one field".into());
      }

      let first = fields.pop()
        .unwrap();

      let inner_type = first.ty;

      Ok(ShrinkwrapInput::Tuple(TupleStruct { ident: ident, inner_type: inner_type }))
    },
    _ => return Err("unsupported data structure type".into())
  }
}
