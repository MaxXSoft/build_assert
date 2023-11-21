#![no_std]

//! `build_assert` allows you to make assertions at build-time.
//!
//! Unlike `assert` and some implementations of compile-time assertions, such
//! as [`static_assertions`](https://docs.rs/static_assertions), `build_assert`
//! works before runtime, and can be used for expressions containing const
//! generics.
//!
//! For example, `build_assert` can be used to:
//!
#![cfg_attr(build = "debug", doc = "```should_panic")]
#![cfg_attr(build = "release", doc = "```compile_fail")]
//! fn foo<const N: usize>() {
//!   # use build_assert::build_assert;
//!   build_assert!(N > 5);
//! }
//!
//! foo::<0>(); // Build-time error!
//! ```
//!
//! The above example will **fail to build in release mode**. Due to the
//! internal implementation, it will **pass the build and panic at runtime**
//! in debug mode.
//!
//! As a comparison, `assert` will only panic at runtime, and static assertion
//! implementations can not be applied to const generics:
//!
//! ```compile_fail
//! macro_rules! static_assert {
//!   ($e:expr) => {
//!     const _: () = core::assert!($e);
//!   };
//! }
//!
//! fn foo<const N: usize>() {
//!   static_assert!(N > 5);
//! }
//! ```
//!
//! An error occurs when compiling the above example:
//!
//! ```text
//! error[E0401]: can't use generic parameters from outer item
//!   --> src/lib.rs:36:18
//!    |
//! 9  | fn foo<const N: usize>() {
//!    |              - const parameter from outer item
//! 10 |   static_assert!(N > 5);
//!    |                  ^ use of generic parameter from outer item
//! ```
//!
//! # Features
//!
//! By default, [`build_assert`] uses inline assembly (i.e. [`core::arch::asm`])
//! to raise build-time errors. If you need to build with this crate on a target
//! that does not support inline assembly (see [the Rust reference]), you can
//! enable the `no_asm` feature.
//!
//! When `no_asm` is enabled, [`build_assert`] raises a link error by referencing
//! an undefined symbol if the assertion fails. By default, the symbol name is
//! `___build_error_impl`. To avoid symbol conflicts, you can set the environment
//! variable `BUILD_ERROR_SYM` to specify a different symbol before building:
//!
//! ```text
//! BUILD_ERROR_SYM=hello cargo build --release
//! ```
//!
//! Note that if the project has been previously built, the build cache should be
//! cleared to ensure this change takes effect.
//!
//! # Under the Hood
//!
//! The [`build_assert`] macro will be expanded to:
//!
//! ```
//! # let cond = false;
//! # macro_rules! build_error { () => {}; }
//! if !cond {
//!   build_error!();
//! }
//! ```
//!
//! In release mode, the condition of `if` expression is expected to be
//! evaluated by the optimizer. If `cond` is `true`, the results of
//! [`build_error`] macro expansion will be optimized away. Otherwise, the
//! expansion results will be retained.
//!
//! On targets that support inline assembly, the [`build_error`] macro will
//! expand to:
//!
//! ```compile_fail
//! core::arch::asm!("build error at file.rs:line:column");
//! ```
//!
//! Since `build` is not a valid instruction on any target, the build will fail.
//!
//! On targets that do not support inline assembly, the [`build_error`] macro
//! will expand to:
//!
//! ```compile_fail
//! extern "Rust" {
//!   fn __build_error_impl() -> !;
//! }
//!
//! unsafe { __build_error_impl() }
//! ```
//!
//! It will occur a link error like this:
//!
//! ```text
//! error: linking with `cc` failed: exit status: 1
//!   |
//!   = note: env -u ...
//!   = note: Undefined symbols for architecture x86_64:
//!             "___build_error_impl", referenced from:
//!                 rust_out::main::... in ... .o
//!           ld: symbol(s) not found for architecture x86_64
//!           clang: error: linker command failed with exit code 1 (use -v to see invocation)
//! ```
//!
//! In debug mode, since the optimizer will not run, the [`build_error`] macro
//! will always be retained. We cannot raise build errors using the above method,
//! otherwise no matter whether the condition is `true` or not, the build will
//! always fail. So the [`build_error`] macro will expand to a [`panic`].
//!
//! # References
//!
//! The idea of `build_assert` macro came from the [Rust for Linux] project.
//! This crate uses a different approach to implement the macro.
//!
//! [the Rust reference]: https://doc.rust-lang.org/nightly/reference/inline-assembly.html
//! [Rust for Linux]: https://rust-for-linux.github.io/docs/kernel/macro.build_assert.html

// TODO:
// 1. doc tests
// 2. more doc comments

#[cfg(all(build = "release", feature = "no_asm"))]
macro_rules! decl_fn {
  ($id:ident) => {
    #[doc(hidden)]
    pub fn $id() -> !;
  };
}

#[cfg(all(build = "release", feature = "no_asm"))]
extern "Rust" {
  env_id::env_id!("BUILD_ERROR_SYM" ?: __build_error_impl => decl_fn);
}

#[cfg(all(build = "release", feature = "no_asm"))]
#[doc(hidden)]
#[inline(always)]
pub fn build_error() {
  unsafe { env_id::env_id!("BUILD_ERROR_SYM" ?: __build_error_impl)() };
}

/// Emits an error at build-time.
#[cfg(build = "debug")]
#[macro_export]
macro_rules! build_error {
  ($($args:tt)*) => {
    core::panic!($($args)*)
  };
}
#[cfg(all(build = "release", not(feature = "no_asm")))]
#[macro_export]
macro_rules! build_error {
  ($($args:tt)*) => {
    unsafe {
      core::arch::asm!(core::concat!(
        "build error at ",
        core::file!(),
        ":",
        core::line!(),
        ":",
        core::column!()
      ))
    }
  };
}
#[cfg(all(build = "release", feature = "no_asm"))]
#[macro_export]
macro_rules! build_error {
  ($($args:tt)*) => {
    $crate::build_error()
  };
}

/// Asserts that a boolean expression is `true` at build-time.
///
/// # Examples
///
#[cfg_attr(build = "debug", doc = "```should_panic")]
#[cfg_attr(build = "release", doc = "```compile_fail")]
/// fn foo<const N: usize>() {
///   # use build_assert::build_assert;
///   build_assert!(N.is_power_of_two());
/// }
///
/// foo::<16>(); // Fine.
/// foo::<15>(); // Fails to compile in release mode, panics in debug mode.
/// ```
#[macro_export]
macro_rules! build_assert {
  ($cond:expr $(,)?) => {
    if !$cond {
      $crate::build_error!(core::concat!("assertion failed: ", core::stringify!($cond)));
    }
  };
  ($cond:expr, $($arg:tt)+) => {
    if !$cond {
      $crate::build_error!($($arg)+);
    }
  };
}

#[macro_export]
macro_rules! build_assert_eq {
  ($left:expr, $right:expr $(,)?) => {
    match (&$left, &$right) {
      (left_val, right_val) => {
        if !(*left_val == *right_val) {
          $crate::build_error!(
            "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
            &*left_val,
            &*right_val,
          );
        }
      }
    }
  };
  ($left:expr, $right:expr, $($arg:tt)+) => {
    match (&$left, &$right) {
      (left_val, right_val) => {
        if !(*left_val == *right_val) {
          $crate::build_error!(
            "assertion `left == right` failed: {}\n  left: {:?}\n right: {:?}",
            core::format_args!($($arg)+),
            &*left_val,
            &*right_val,
          );
        }
      }
    }
  };
}

#[macro_export]
macro_rules! build_assert_ne {
  ($left:expr, $right:expr $(,)?) => {
    match (&$left, &$right) {
      (left_val, right_val) => {
        if *left_val == *right_val {
          $crate::build_error!(
            "assertion `left != right` failed\n  left: {:?}\n right: {:?}",
            &*left_val,
            &*right_val,
          );
        }
      }
    }
  };
  ($left:expr, $right:expr, $($arg:tt)+) => {
    match (&$left, &$right) {
      (left_val, right_val) => {
        if *left_val == *right_val {
          $crate::build_error!(
            "assertion `left != right` failed: {}\n  left: {:?}\n right: {:?}",
            core::format_args!($($arg)+),
            &*left_val,
            &*right_val,
          );
        }
      }
    }
  };
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_build_assert() {
    build_assert!(true);
  }

  #[cfg(build = "debug")]
  #[test]
  #[should_panic(expected = "assertion failed: false")]
  fn test_build_assert_fail() {
    build_assert!(false);
  }

  fn assert_const<const N: usize>() {
    build_assert!(N > 10, "N must be greater than 10, got {}", N);
  }

  #[test]
  fn test_assert_const() {
    assert_const::<11>();
  }

  #[cfg(build = "debug")]
  #[test]
  #[should_panic(expected = "N must be greater than 10, got 10")]
  fn test_assert_const_fail() {
    assert_const::<10>();
  }

  #[test]
  fn test_build_assert_eq() {
    build_assert_eq!(1, 1);
  }

  #[cfg(build = "debug")]
  #[test]
  #[should_panic(expected = "assertion `left == right` failed\n  left: 1\n right: 2")]
  fn test_build_assert_eq_fail() {
    build_assert_eq!(1, 2);
  }

  fn assert_const_eq<const A: usize, const B: usize>() {
    build_assert_eq!(A, B, "A must be equal to B, got {A} and {B}");
  }

  #[test]
  fn test_assert_const_eq() {
    assert_const_eq::<1, 1>();
  }

  #[cfg(build = "debug")]
  #[test]
  #[should_panic(
    expected = "assertion `left == right` failed: A must be equal to B, got 1 and 2\n  left: 1\n right: 2"
  )]
  fn test_assert_const_eq_fail() {
    assert_const_eq::<1, 2>();
  }

  #[test]
  fn test_build_assert_ne() {
    build_assert_ne!(1, 2);
  }

  #[cfg(build = "debug")]
  #[test]
  #[should_panic(expected = "assertion `left != right` failed\n  left: 1\n right: 1")]
  fn test_build_assert_ne_fail() {
    build_assert_ne!(1, 1);
  }

  fn assert_const_ne<const A: usize, const B: usize>() {
    build_assert_ne!(A, B, "A must not be equal to B, got {} and {}", A, B);
  }

  #[test]
  fn test_assert_const_ne() {
    assert_const_ne::<1, 2>();
  }

  #[cfg(build = "debug")]
  #[test]
  #[should_panic(
    expected = "assertion `left != right` failed: A must not be equal to B, got 1 and 1\n  left: 1\n right: 1"
  )]
  fn test_assert_const_ne_fail() {
    assert_const_ne::<1, 1>();
  }
}
