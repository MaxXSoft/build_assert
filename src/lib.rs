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
            format!($($arg)+),
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
            format!($($arg)+),
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

  #[test]
  fn test_build_assert_eq() {
    build_assert_eq!(1, 1);
  }

  #[cfg(debug_assertions)]
  #[test]
  #[should_panic(expected = "assertion `left == right` failed\n  left: 1\n right: 2")]
  fn test_build_assert_eq_fail() {
    build_assert_eq!(1, 2);
  }

  fn assert_const_eq<const A: usize, const B: usize>() {
    build_assert_eq!(A, B, "A must be equal to B, got {} and {}", A, B);
  }

  #[test]
  fn test_assert_const_eq() {
    assert_const_eq::<1, 1>();
  }

  #[cfg(debug_assertions)]
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

  #[cfg(debug_assertions)]
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

  #[cfg(debug_assertions)]
  #[test]
  #[should_panic(
    expected = "assertion `left != right` failed: A must not be equal to B, got 1 and 1\n  left: 1\n right: 1"
  )]
  fn test_assert_const_ne_fail() {
    assert_const_ne::<1, 1>();
  }
}
