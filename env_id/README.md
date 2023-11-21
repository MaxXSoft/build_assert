# env_id

Use environment variables as identifiers.

## Usage

Add `env_id` to your project by running `cargo add`:

```
cargo add env_id
```

## Examples

```rust
let env_id!("CARGO_CRATE_NAME") = 1;
dbg!(env_id!("CARGO_CRATE_NAME"));
```

Or you can provide a default value:

```rust
let env_id!("HELLO" ?: hello) = 1;
dbg!(env_id!("HELLO" ?: hello));
```

This may be useful when you want to let users specify the name of a public item, but the following code doesn't compile:

```rust
pub const env_id!("HELLO" ?: hello): usize = 1;
```

You can use another macro to do the same thing:

```rust
macro_rules! def_const {
  ($id:ident) => {
    pub const $id: usize = 1;
  };
}

env_id!("HELLO"?: hello => def_const);
```

## License

Copyright (C) 2023 MaxXing. Licensed under either of Apache 2.0 or MIT at your option.
