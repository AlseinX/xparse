use alloc::boxed::Box;
use core::fmt::{Debug, Display};

use thiserror_no_std::Error;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("mismatch")]
    Mismatch,

    #[error("mismatching {0}")]
    NamedMismatch(&'static str),

    #[error(transparent)]
    Hard(HardError),
}

#[derive(Debug, Error)]
pub enum HardError {
    #[error("incomplete {name} at {position}")]
    Incomplete { position: usize, name: &'static str },

    #[error("incomplete {name} at {position}, expecting {component_name}")]
    NamedIncomplete {
        position: usize,
        name: &'static str,
        component_name: &'static str,
    },

    #[error(transparent)]
    Other(#[from] Box<dyn DynError + Send>),
}

impl<T: Into<HardError>> From<T> for Error {
    fn from(value: T) -> Self {
        Self::Hard(value.into())
    }
}

pub trait DynError: Debug + Display {}
impl<T: Debug + Display + ?Sized> DynError for T {}
