# Changelog

## [Unreleased]

* Added a changelog
* Added support for structs with lifetimes and generic parameters

## [v0.1.0] -- 2018-02-06

* Split out derivation of mutable traits (`DerefMut`, `BorrowMut`, `AsMut`) into
  separate derive trait `ShrinkwrapMut`

## [v0.0.2] -- 2018-02-04

* Fixed typoes in documentation -- no functionality changes

## [v0.0.1] -- 2018-02-04

* Initial release -- implemented `#[derive(Shrinkwrap)]` to auto-derive
  `Deref`, `DerefMut`, `Borrow`, `BorrowMut`, `AsRef`, and `AsMut`.
