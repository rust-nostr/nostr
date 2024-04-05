# Code style

This is a description of a coding style that every contributor **must** follow.
Please, read the whole document before you start pushing code.

## Generics

All trait bounds should be written in `where`:

```rust
// GOOD
pub fn new<N, T, P, E>(user_id: i32, name: N, title: T, png_sticker: P, emojis: E) -> Self
where
    N: Into<String>,
    T: Into<String>,
    P: Into<InputFile>,
    E: Into<String>,
{ ... }

// BAD
pub fn new<N: Into<String>,
           T: Into<String>,
           P: Into<InputFile>,
           E: Into<String>>
    (user_id: i32, name: N, title: T, png_sticker: P, emojis: E) -> Self { ... }
```

```rust
// GOOD
impl<T> Trait for Wrap<T>
where
    T: Trait
{ ... }

// BAD
impl<T: Trait> Trait for Wrap<T> { ... }
```

## Use `Self` where possible

When referring to the type for which block is implemented, prefer using `Self`, rather than the name of the type:

```rust
impl ErrorKind {
    // GOOD
    fn print(&self) {
        Self::Io => println!("Io"),
        Self::Network => println!("Network"),
        Self::Json => println!("Json"),
    }

    // BAD
    fn print(&self) {
        ErrorKind::Io => println!("Io"),
        ErrorKind::Network => println!("Network"),
        ErrorKind::Json => println!("Json"),
    }
}
```

```rust
impl<'a> AnswerCallbackQuery<'a> {
    // GOOD
    fn new<C>(bot: &'a Bot, callback_query_id: C) -> Self
    where
        C: Into<String>,
    { ... }

    // BAD
    fn new<C>(bot: &'a Bot, callback_query_id: C) -> AnswerCallbackQuery<'a>
    where
        C: Into<String>,
    { ... }
}
```

**Rationale:** `Self` is generally shorter and it's easier to copy-paste code or rename the type.

## Deriving traits

Derive `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq` and `Hash` for public types when possible (in this order).

**Rationale:** these traits can be useful for users and can be implemented for most types.

Derive `Default` when there is a reasonable default value for the type.

## Full paths for logging

Always write `tracing::<op>!(...)` instead of importing `use tracing::<op>;` and invoking `<op>!(...)`.

```rust
// GOOD
tracing::warn!("Everything is on fire");

// BAD
use tracing::warn;

warn!("Everything is on fire");
```

**Rationale:**
- Less polluted import blocks
- Uniformity

## `&str` -> `String` conversion

Prefer using `.to_string()` or `.to_owned()`, rather than `.into()`, `String::from`, etc.

**Rationale:** uniformity, intent clarity.

## Order of imports

```rust
// First core, alloc and/or std
use core::fmt;
use std::{...};

// Second, external crates (both crates.io crates and other rust-analyzer crates).
use crate_foo::{ ... };
use crate_bar::{ ... };

// If applicable, the current sub-modules
mod x;
mod y;

// Finally, the internal crate modules and submodules
use crate::{};
use super::{};

// Re-exports are treated as item definitions rather than imports, so they go
// after imports and modules. Use them sparingly.
pub use crate::x::Z;
```

## Import Style

When implementing traits from `core::fmt`/`std::fmt` import the module:

```rust
// GOOD
use core::fmt;

impl fmt::Display for RenameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { .. }
}

// BAD
impl core::fmt::Display for RenameError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { .. }
}
```

## If-let

Avoid the `if let ... { } else { }` construct if possible, use `match` instead:

```rust
// GOOD
match ctx.expected_type.as_ref() {
    Some(expected_type) => completion_ty == expected_type && !expected_type.is_unit(),
    None => false,
}

// BAD
if let Some(expected_type) = ctx.expected_type.as_ref() {
    completion_ty == expected_type && !expected_type.is_unit()
} else {
    false
}
```

Use `if let ... { }` when a match arm is intentionally empty:

```rust
// GOOD
if let Some(expected_type) = this.as_ref() {
    // Handle it
}

// BAD
match this.as_ref() {
    Some(expected_type) => {
        // Handle it
    },
    None => (),
}
```