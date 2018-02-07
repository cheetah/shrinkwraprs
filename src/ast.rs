//! We want to make sure that the struct that our caller passes us is in the
//! right form. However, we don't want to clutter up our code generation
//! logic with lots of error handling. So instead, we take in our `DeriveInput`
//! and do all the error handling in one place, transforming it into an AST
//! specific to our crate if it's valid.

use syn;

use itertools::Itertools;

type Fields = Vec<syn::Field>;

pub struct StructDetails {
  pub ident: syn::Ident,
  pub visibility: syn::Visibility
}

/// Represents a 1-tuple struct.
pub struct Tuple {
  pub inner_type: syn::Type
}

/// Represents an n-tuple struct, with one of the elements designated
/// as the one we should deref to.
pub struct NaryTuple {
  pub inner_field_index: syn::Index,
  pub inner_type: syn::Type
}

/// Represents a normal struct with a single named field.
pub struct Single {
  pub inner_field: syn::Ident,
  pub inner_type: syn::Type,
  pub inner_visibility: syn::Visibility
}

/// Represents a normal struct with multiple named fields, one of which we
/// should deref to.
pub struct Multi {
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

pub fn validate_derive_input(input: syn::DeriveInput) -> (StructDetails, ShrinkwrapInput) {
  // Note that `unwrap()`s and `panic()`s are totally fine here, since we're
  // inside a procedural macro; panics happen at compile time

  use syn::{DeriveInput, DataStruct, FieldsUnnamed, FieldsNamed};
  use syn::Data::{Struct, Enum, Union};
  use syn::Fields::{Named, Unnamed};

  let DeriveInput { attrs: _attrs, vis, ident, generics, data, .. } = input;

  if !generics.params.is_empty() {
    panic!("currently, shrinkwraprs does not support structs with lifetimes or generics");
  }

  let details = StructDetails { ident: ident, visibility: vis };

  let input = match data {
    Struct(DataStruct { fields: Unnamed(FieldsUnnamed { unnamed: fields, .. }), .. }) => {
      let fields = fields.into_iter().collect_vec();
      validate_tuple(fields)
    },
    Struct(DataStruct { fields: Named(FieldsNamed { named: fields, .. }), .. }) => {
      let fields = fields.into_iter().collect_vec();
      validate_struct(fields)
    },
    Struct(..) =>
      panic!("shrinkwraprs needs a struct with at least one field!"),
    Enum(..) =>
      panic!("shrinkwraprs does not support enums"),
    Union(..) =>
      panic!("shrinkwraprs does not support C-style unions")
  };

  (details, input)
}

fn is_marked(field: &syn::Field) -> bool {
  use syn::{Meta, MetaList, NestedMeta};

  let mut attrs = field.attrs.iter();

  attrs.any(|attr| {
    let meta = attr.interpret_meta();

    if let Some(Meta::List(MetaList { ident, nested, ..})) = meta {
      let nested_metas: Option<(NestedMeta,)> = nested.into_iter()
        .collect_tuple();
      let is_main_field = match nested_metas {
        Some((NestedMeta::Meta(Meta::Word(word)),)) => &word == "main_field",
        _ => false
      };

      &ident == "shrinkwrap" && is_main_field
    } else {
      false
    }
  })
}

/// Only a single field, out of all a struct's fields, can be marked as
/// the main field that we deref to. So let's find that field.
/// We also return the 0-based number of the marked field.
fn find_marked_field(fields: Fields) -> ((usize, syn::Field), Fields) {
  let (marked, unmarked) = fields.into_iter()
    .enumerate()
    .partition::<Vec<_>, _>(|&(_, ref field)| is_marked(field));
  let marked_len = marked.len();
  let single: Option<(_,)> = marked.into_iter()
    .collect_tuple();

  match (single, unmarked.len()) {
    (Some((field,)), _) => {
      let unmarked = unmarked.into_iter()
        .map(|(_, field)| field)
        .collect_vec();

      (field, unmarked)
    }
    (None, 1) => {
      let single: (_,) = unmarked.into_iter()
        .collect_tuple()
        .unwrap();

      (single.0, vec![])
    },
    _ => if marked_len == 0 {
      panic!("halp! shrinkwraprs doesn't know which field you want this struct to convert to.
Did you forget to mark a field with #[shrinkwrap(main_field)]?");
    } else {
      panic!("halp! shrinkwraprs doesn't know which field you want this struct to convert to.
Did you accidentally mark more than one field with #[shrinkwrap(main_field)]?");
    }
  }
}

fn validate_tuple(fields: Fields) -> ShrinkwrapInput {
  if fields.len() == 0 {
    panic!("shrinkwraprs requires tuple structs to have at least one field");
  }

  let (marked, unmarked) = find_marked_field(fields);

  if unmarked.len() == 0 {
    ShrinkwrapInput::Tuple(Tuple {
      inner_type: marked.1.ty
    })
  } else {
    ShrinkwrapInput::NaryTuple(NaryTuple {
      inner_field_index: marked.0.into(),
      inner_type: marked.1.ty
    })
  }
}

fn validate_struct(fields: Fields) -> ShrinkwrapInput {
  if fields.len() == 0 {
    panic!("shrinkwraprs requires structs to have at least one field");
  }

  let (marked, unmarked) = find_marked_field(fields);
  let ident = marked.1.ident
    .unwrap();
  let ty = marked.1.ty;
  let vis = marked.1.vis;

  if unmarked.len() == 0 {
    ShrinkwrapInput::Single(Single {
      inner_field: ident,
      inner_type: ty,
      inner_visibility: vis
    })
  } else {
    ShrinkwrapInput::Multi(Multi {
      inner_field: ident,
      inner_type: ty,
      inner_visibility: vis
    })
  }
}

#[cfg(test)]
mod tests {
  use syn;
  use itertools::Itertools;

  use super::*;

  #[test]
  fn test_field_attribute_found() {
    let input = r"
      struct Foo {
        field1: u32,
        #[shrinkwrap(main_field)]
        field2: u32
      }
    ";

    let strct: syn::DeriveInput = syn::parse_str(input)
      .unwrap();

    match strct.data {
      syn::Data::Struct(syn::DataStruct { fields, .. }) => {
        let marked = fields.into_iter()
          .filter(|field| is_marked(field));
        let field: (&syn::Field,) = marked
          .collect_tuple()
          .unwrap();
        let ident = field.0.ident
          .unwrap();

        assert_eq!(&ident, "field2");
      },
      _ => panic!()
    }
  }

  #[test]
  fn test_field_attribute_not_found() {
    let input = r"
      struct Foo {
        field1: u32,
        field2: u32
      }
    ";

    let strct: syn::DeriveInput = syn::parse_str(input)
      .unwrap();

    match strct.data {
      syn::Data::Struct(syn::DataStruct { fields, .. }) => {
        let marked = fields.into_iter()
          .filter(|field| is_marked(field))
          .collect_vec();
        assert_eq!(marked.len(), 0);
      },
      _ => panic!()
    }
  }
}
