//! Use environment variables as identifiers.
//!
//! # Examples
//!
//! ```
//! # use env_id::env_id;
//! let env_id!("CARGO_CRATE_NAME") = 1;
//! dbg!(env_id!("CARGO_CRATE_NAME"));
//! ```
//!
//! Or you can provide a default value:
//!
//! ```
//! # use env_id::env_id;
//! let env_id!("HELLO" ?: hello) = 1;
//! dbg!(env_id!("HELLO" ?: hello));
//! ```
//!
//! This may be useful when you want to let users specify the name of a
//! public item, but the following code doesn't compile:
//!
//! ```compile_fail
//! # fn main() {}
//! # use env_id::env_id;
//! pub const env_id!("HELLO" ?: hello): usize = 1;
//! ```
//!
//! You can use another macro to do the same thing:
//!
//! ```
//! # fn main() {}
//! # use env_id::env_id;
//! macro_rules! def_const {
//!   ($id:ident) => {
//!     pub const $id: usize = 1;
//!   };
//! }
//!
//! env_id!("HELLO"?: hello => def_const);
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{
  parse::{Parse, ParseStream},
  Error, Ident, LitStr, Result, Token,
};

/// Uses the given environment variable as an identifier.
/// 
/// See the [module-level documentation](self) for more information.
/// 
/// # Definition
/// 
/// ```
/// macro_rules! env_id {
///   ($name:literal) => { ... };
///   ($name:literal ?: $default_id:ident) => { ... };
///   ($name:literal ?: $default_id:ident => $apply_to:ident) => { ... };
/// }
/// ```
#[proc_macro]
pub fn env_id(tokens: TokenStream) -> TokenStream {
  match parse_env_id(tokens) {
    Ok(tokens) => tokens,
    Err(err) => err.to_compile_error().into(),
  }
}

/// Parses the `env_id` macro.
fn parse_env_id(tokens: TokenStream) -> Result<TokenStream> {
  // Parse macro input.
  let env_id: EnvId = syn::parse(tokens)?;
  // Get value of the environment variable.
  let ident = match std::env::var(env_id.name.value()) {
    Ok(value) => Ident::new(&value, env_id.name.span()),
    Err(e) => env_id.default_id.map(|d| d.ident).ok_or(Error::new(
      env_id.name.span(),
      format!("failed to get environment variable: {e}"),
    ))?,
  };
  // Generate result.
  Ok(TokenStream::from(if let Some(apply_to) = env_id.apply_to {
    let m = apply_to.ident;
    quote!(#m!(#ident);)
  } else {
    quote!(#ident)
  }))
}

/// AST of the `env_id` macro.
struct EnvId {
  name: LitStr,
  default_id: Option<DefaultId>,
  apply_to: Option<ApplyTo>,
}

impl Parse for EnvId {
  fn parse(input: ParseStream) -> Result<Self> {
    // Parse literal string.
    let name = input.parse()?;
    // Parse the optional default identifier.
    let default_id = if input.peek(Token![?]) {
      Some(input.parse::<DefaultId>()?)
    } else {
      None
    };
    // Parse the optional apply-to macro.
    let apply_to = if input.peek(Token![=>]) {
      Some(input.parse::<ApplyTo>()?)
    } else {
      None
    };
    Ok(Self {
      name,
      default_id,
      apply_to,
    })
  }
}

/// Default identifier.
struct DefaultId {
  _question: Token![?],
  _colon: Token![:],
  ident: Ident,
}

impl Parse for DefaultId {
  fn parse(input: ParseStream) -> Result<Self> {
    Ok(Self {
      _question: input.parse()?,
      _colon: input.parse()?,
      ident: input.parse()?,
    })
  }
}

/// Apply-to macro.
struct ApplyTo {
  _fat_arrow: Token![=>],
  ident: Ident,
}

impl Parse for ApplyTo {
  fn parse(input: ParseStream) -> Result<Self> {
    Ok(Self {
      _fat_arrow: input.parse()?,
      ident: input.parse()?,
    })
  }
}
