# shrinkwraprs

Making wrapper types allows us to give more compile-time
guarantees about our code being correct:

```rust
// Now we can't mix up widths and heights; the compiler will yell at us!
struct Width(i64);
struct Height(i64);
```

But... they're kind of a pain to work with. If you ever need to get at
that wrapped `i64`, you need to constantly pattern-match back and forth
to wrap and unwrap the values.

`shrinkwraprs` aims to alleviate this pain by allowing you to derive
implementations of various conversion traits by attaching
`#[derive(Shrinkwrap)]`.

## Traits implemented

Currently, `shrinkwraprs` derives the following traits for all structs:

* `AsRef<InnerType>`
* `AsMut<InnerType>`
* `Borrow<InnerType>`
* `BorrowMut<InnerType>`
* `Deref<Target=InnerType>`
* `DerefMut<Target=InnerType>`

## Cool, how do I use it?

First, add `shrinkwraprs` as a dependency in your `Cargo.toml`:

```toml
[dependencies]

shrinkwraprs = "0.0.1"
```

Then, just slap a `#[derive(Shrinkwrap)]` on any structs you want
convenient-ified:

```rust
#[macro_use] extern crate shrinkwraprs;

#[derive(Shrinkwrap)]
struct Email(String);

fn main() {
  let email = Email("chiya+snacks@natsumeya.jp".into());

  let is_discriminated_email =
    (*email).contains("+");  // Woohoo, we can use the email like a string!

  /* ... */
}
```

If you have multiple fields, but there's only one field you want to be able
to deref/borrow as, mark it with `#[shrinkwrap(main_field)]`:

```rust
#[derive(Shrinkwrap)]
struct Email {
  spamminess: f64,
  #[shrinkwrap(main_field)] addr: String
}

#[derive(Shrinkwrap)]
struct CodeSpan(u32, u32, #[shrinkwrap(main_field)] Token);
```
