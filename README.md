# build_assert

`build_assert` allows you to make assertions at build-time.

Unlike `assert` and some implementations of compile-time assertions, such as [`static_assertions`](https://docs.rs/static_assertions), `build_assert` works before runtime, and can be used for expressions containing const generics.

## Usage

Add `build_assert` to your project by running `cargo add`:

```
cargo add build_assert
```

## Examples

```rust
fn foo<const N: usize>() {
  build_assert!(N > 5);
}

foo::<10>(); // Fine.
foo::<0>();  // Fails to compile.
```

The above example will **fail to build in release mode**. Due to the internal implementation, it will **pass the build and panic at runtime** in debug mode.

As a comparison, `assert` will only panic at runtime, and static assertion implementations can not be applied to const generics:

```rust
macro_rules! static_assert {
  ($e:expr) => {
    const _: () = core::assert!($e);
  };
}

fn foo<const N: usize>() {
  static_assert!(N > 5);
}
```

An error occurs when compiling the above example:

```
error[E0401]: can't use generic parameters from outer item
  --> src/lib.rs:36:18
   |
9  | fn foo<const N: usize>() {
   |              - const parameter from outer item
10 |   static_assert!(N > 5);
   |                  ^ use of generic parameter from outer item
```

## Features

By default, `build_assert` uses inline assembly (i.e. `core::arch::asm`) to raise build-time errors. If you need to build with this crate on a target that does not support inline assembly (see [the Rust reference](https://doc.rust-lang.org/nightly/reference/inline-assembly.html)), you can enable the `no_asm` feature.

When `no_asm` is enabled, `build_assert` raises a link error by referencing an undefined symbol if the assertion fails. By default, the symbol name is `__build_error_impl`. To avoid symbol conflicts, you can set the environment variable `BUILD_ERROR_SYM` to specify a different symbol before building:

```text
BUILD_ERROR_SYM=hello cargo build --release
```

Note that if the project has been previously built, the build cache should be cleared to ensure this change takes effect.

## Under the Hood

The `build_assert` macro will be expanded to:

```rust
if !cond {
  build_error!();
}
```

In release mode, the condition of `if` expression is expected to be evaluated by the optimizer. If `cond` is `true`, the results of
`build_error` macro expansion will be optimized away. Otherwise, the expansion results will be retained.

On targets that support inline assembly, the `build_error` macro will expand to:

```rust
core::arch::asm!("build error at file.rs:line:column");
```

Since `build` is not a valid instruction on any target, the build will fail.

On targets that do not support inline assembly, the `build_error` macro will expand to:

```rust
extern "Rust" {
  fn __build_error_impl() -> !;
}

unsafe { __build_error_impl() }
```

It will occur a link error like this:

```text
error: linking with `cc` failed: exit status: 1
  |
  = note: env -u ...
  = note: /usr/bin/ld: ... .o: in function `rust_out::main::...':
          ... .rs:6: undefined reference to `__build_error_impl'
          collect2: error: ld returned 1 exit status

  = note: ...
```

In debug mode, since the optimizer will not run, the `build_error` macro will always be retained. We cannot raise build errors using the above method, otherwise no matter whether the condition is `true` or not, the build will always fail. So the `build_error` macro will expand to a `panic`.

## References

The idea of `build_assert` macro came from the [Rust for Linux](https://rust-for-linux.github.io/docs/kernel/macro.build_assert.html) project.
This crate uses a different approach to implement the macro.

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## License

Copyright (C) 2023 MaxXing. Licensed under either of [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
