// TODO:
// 1. no std
// 2. reference
// 3. doc tests
// 4. more doc comments

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! build_error {
  ($($args:tt)+) => {
    std::panic!($($args)+)
  };
}

#[cfg(not(debug_assertions))]
extern "Rust" {
  #[doc(hidden)]
  pub fn __build_error_impl() -> !;
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! build_error {
  ($($args:tt)+) => {
    unsafe { $crate::__build_error_impl() }
  };
}

/// Asserts that a boolean expression is true at build-time.
///
/// # Examples
///
/// ```
/// fn foo<const N: usize>() {
///   build_assert::build_assert!(N.is_power_of_two());
/// }
///
/// foo::<16>();    // Fine.
/// // foo::<15>(); // Fails to compile in release mode, panics in debug mode.
/// ```
#[macro_export]
macro_rules! build_assert {
  ($cond:expr $(,)?) => {
    if !$cond {
      $crate::build_error!(std::concat!("assertion failed: ", std::stringify!($cond)));
    }
  };
  ($cond:expr, $($arg:tt)+) => {
    if !$cond {
      $crate::build_error!($($arg)+);
    }
  };
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_build_assert() {
    build_assert!(true);
  }

  #[cfg(debug_assertions)]
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

  #[cfg(debug_assertions)]
  #[test]
  #[should_panic(expected = "N must be greater than 10, got 10")]
  fn test_assert_const_fail() {
    assert_const::<10>();
  }
}
