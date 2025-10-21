#![doc = include_str!("../README.md")]

#[doc(hidden)]
pub mod de;

mod error;

pub use error::ServiceConfError;
pub use serviceconf_derive::ServiceConf;

// Re-export for macro-generated code
#[doc(hidden)]
pub use anyhow;
