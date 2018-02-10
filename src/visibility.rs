//! We want to make sure that providing mutable traits doesn't accidentally
//! leak the internal implementation details of a shrinkwrapped type.
//!
//! To do that, we need to make sure that the inner field has the same
//! visibility as the shrinkwrapped struct itself. If it doesn't, we can
//! give the user an error and refuse to generate implementations.

use syn;

use itertools::Itertools;

// When checking for visibility containment, we can make use of the guarantee
// that the langauge provides us that any visibility path must be a parent
// module of the current one. This means, for instance, that we don't have
// to worry about the possibility of something like `pub(in self::some_mod)`;
// we also know that if we see a `pub(in super::some_mod)`, we can assume
// that `some_mod` is the module that the shrinkwrapped type is in.
//
// This doesn't help us in degenerate cases, like one path is
// `pub(in ::a::b::c)` and the other is `pub(super)`, and it turns out that
// those two are the same module, but theoretically it can allow us to determine
// more cases.

#[derive(PartialEq, Debug)]
pub enum PathComponent {
  /// Effectively, this means private.
  Inherited,
  Pub,
  Crate,
  InSelf,
  InSuper,
  Mod(String)
}

pub enum FieldVisibility {
  /// The inner field is *at least* as visible as its containing struct.
  Visible,
  /// The inner field is less visible than its containing struct.
  Restricted,
  /// We can't figure out how the visibilities relate, probably due to the
  /// paths starting at different points (e.g. one is self and the other
  /// is ::a::b::c)
  CantDetermine
}

fn to_path(path: &syn::Visibility) -> Vec<PathComponent> {
  use syn::Visibility::*;

  match path {
    &Public(..) => vec![ PathComponent::Pub ],
    &Crate(..) => vec![ PathComponent::Pub, PathComponent::Crate ],
    &Inherited => vec![ PathComponent::Inherited ],
    &Restricted(ref vis) => to_path_restricted(&vis.path)
  }
}

fn to_path_restricted(path: &syn::Path) -> Vec<PathComponent> {
  let segments = path.segments.iter()
    .map(|path_segment| path_segment.ident.to_string())
    .collect_vec();

  match segments.split_first() {
    None => vec![],
    Some((ident, rest)) => {
      let mut result;

      if *ident == "self" {
        result = vec![ PathComponent::InSelf ];
      } else if *ident == "super" {
        result = vec![ PathComponent::InSuper ];
      } else {
        // We add these components in non-self/super paths to allow us to
        // match them up with visibilities like `pub` and `pub(crate)`.
        result = vec![ PathComponent::Pub, PathComponent::Crate, PathComponent::Mod(ident.to_string()) ];
      }

      let rest = rest.iter()
        .map(|ident| PathComponent::Mod(ident.to_string()));

      result.extend(rest);

      result
    }
  }
}

#[cfg(test)]
mod path_convert_tests {
  use std::convert::From;

  use syn::{self, Visibility};

  use super::{PathComponent, to_path};

  static VIS1: &'static str = "pub";
  static VIS2: &'static str = "pub(crate)";
  static VIS3: &'static str = "";
  static VIS4: &'static str = "pub(self)";
  static VIS5: &'static str = "pub(super)";
  static VIS6: &'static str = "pub(in ::a::b::c)";
  static VIS7: &'static str = "pub(in ::super::b)";

  impl<'a> From<&'a str> for PathComponent {
    fn from(input: &'a str) -> Self {
      PathComponent::Mod(input.to_string())
    }
  }

  macro_rules! vis_test {
    ($input:ident, $($component:expr);+) => {{
      let vis: Visibility = syn::parse_str($input)
        .expect("path input is structured incorrectly!");
      let vis = to_path(&vis);

      let expected = vec![ $($component.into()),+ ];

      assert_eq!(&vis, &expected);
    }}
  }

  #[test]
  fn test_vis1() {
    vis_test!(VIS1, PathComponent::Pub);
  }

  #[test]
  fn test_vis2() {
    vis_test!(VIS2, PathComponent::Pub; PathComponent::Crate);
  }

  #[test]
  fn test_vis3() {
    vis_test!(VIS3, PathComponent::Inherited);
  }

  #[test]
  fn test_vis4() {
    vis_test!(VIS4, PathComponent::InSelf);
  }

  #[test]
  fn test_vis5() {
    vis_test!(VIS5, PathComponent::InSuper);
  }

  #[test]
  fn test_vis6() {
    vis_test!(VIS6, PathComponent::Pub; PathComponent::Crate; "a"; "b"; "c");
  }

  #[test]
  fn test_vis7() {
    vis_test!(VIS7, PathComponent::InSuper; "b");
  }
}
