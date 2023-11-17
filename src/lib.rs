#![no_std]

// TODO:
// 1. reference
// 2. doc tests
// 3. more doc comments

#[cfg(build = "release")]
extern "Rust" {
  #[doc(hidden)]
  pub fn __build_error_impl() -> !;
}

/// Emits an error at build-time.
#[cfg(build = "debug")]
#[macro_export]
macro_rules! build_error {
  ($($args:tt)*) => {
    core::panic!($($args)*)
  };
}
#[cfg(build = "release")]
#[macro_export]
macro_rules! build_error {
  ($($args:tt)*) => {
    unsafe { $crate::__build_error_impl() }
  };
}

/// Asserts that a boolean expression is true at build-time.
///
/// # Examples
///
#[cfg_attr(build = "debug", doc = "```should_panic")]
#[cfg_attr(build = "release", doc = "```compile_fail")]
/// fn foo<const N: usize>() {
///   build_assert::build_assert!(N.is_power_of_two());
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
