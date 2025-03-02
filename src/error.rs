#![allow(clippy::module_name_repetitions)]

use std::alloc::LayoutError;

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum AllocationError {
    #[error("An arithmetic error occured.")]
    ArithmeticError,
    #[error(transparent)]
    LayoutError(#[from] LayoutError),
    #[error("Process ran out of memory.")]
    OutOfMemory,
}

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum DeallocationError {
    #[error("This memory was already freed.")]
    DoubleFree,
    #[error("The pointer provided was not properly aligned.")]
    ImproperAlignment,
    #[error("Tried to free memory not allocated by this crate.")]
    InvalidAllocation,
    #[error(transparent)]
    LayoutError(#[from] LayoutError),
    #[error("Tried to free a null pointer.")]
    NullPtr,
}
