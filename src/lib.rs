#![cfg_attr(not(test), no_std)]
extern crate alloc;

mod parse;

pub mod source;
pub use parse::*;
#[cfg(feature = "async")]
pub use source::AsyncSource;
pub use source::{Source, SourceBase};
mod error;
pub use error::*;
pub mod ops;
mod tuple;
#[cfg(feature = "async")]
pub use parse::parser;
pub use tuple::Concat;

extern crate self as xparse;

#[cfg(test)]
mod json;
