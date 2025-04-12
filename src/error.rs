#![allow(clippy::module_name_repetitions)]

use std::alloc::LayoutError;

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum AllocationError {
    #[error("An arithmetic error occured.")]
    ArithmeticError,
    #[error("The allocated memory was not properly aligned.")]
    ImproperAlignment,
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

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum ReallocationError {
    #[error(transparent)]
    AllocationError(#[from] AllocationError),
    #[error(transparent)]
    DeallocationError(#[from] DeallocationError),
    #[error("Deallocation failed, cleanup failed: {0}, {1}")]
    FreeFailedTwice(DeallocationError, DeallocationError),
    #[error("Refusing to realloc an improperly aligned pointer.")]
    ImproperAlignment,
    #[error("Refusing to realloc a pointer not allocated by ps-alloc.")]
    InvalidPointer,
    #[error("Refusing to realloc a freed pointer.")]
    UseAfterFree,
}
