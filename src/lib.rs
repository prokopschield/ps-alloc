#![allow(clippy::not_unsafe_ptr_arg_deref)]

mod error;

use std::alloc::Layout;

pub use error::*;

pub const HEADER_SIZE: usize = std::mem::size_of::<AllocationHeader>();
pub const MARKER_FREE: [u8; 8] = *b"Fr33Mmry";
pub const MARKER_USED: [u8; 8] = *b"U53dMmry";

#[repr(align(16))]
struct AllocationHeader {
    marker: [u8; 8],
    size: usize,
}

/// - A reasonably safe implementation of `alloc`.
/// - Memory allocated by this function must be freed by this crate's `free`.
/// - Caller guarantees `free` is called before the returned pointer goes out of scope.
/// # Errors
/// - `Err(ArithmeticError)` on integer overflow.
/// - `Err(ImproperAlignment)` if the global allocator returns a misaligned pointer.
/// - `Err(LayoutError)` if [`ALIGNMENT`] isn't a power of 2 or the computed size is not aligned.
/// - `Err(OutOfMemory)` if `alloc()` returns a `nullptr`.
#[allow(clippy::cast_ptr_alignment)]
pub fn alloc(size: usize) -> Result<*mut u8, AllocationError> {
    let size = size
        .checked_add(HEADER_SIZE)
        .ok_or(AllocationError::ArithmeticError)?
        .checked_next_multiple_of(HEADER_SIZE)
        .ok_or(AllocationError::ArithmeticError)?;

    let layout = Layout::from_size_align(size, HEADER_SIZE)?;

    let ptr = unsafe { std::alloc::alloc(layout) };

    if ptr.is_null() {
        return Err(AllocationError::OutOfMemory);
    }

    if 0 != (ptr as usize % HEADER_SIZE) {
        unsafe { std::alloc::dealloc(ptr, layout) };

        return Err(AllocationError::ImproperAlignment);
    }

    let header = unsafe { &mut *(ptr.cast::<AllocationHeader>()) };

    header.marker = MARKER_USED;
    header.size = size;

    let ptr = unsafe { ptr.add(HEADER_SIZE) };

    Ok(ptr)
}

/// - A reasonably safe implementation of `free`.
/// - This function will free a pointer allocated by `alloc`.
/// - Caller guarantees that the provided pointer was allocated by this crate's `alloc` function.
/// - Providing `NULL` is safe and will return `Err(DeallocationError::NullPtr)`.
/// - Providing any other pointer causes undefined behaviour.
/// # Errors
/// - Returns `Err(DeallocationError)` if a safety check fails.
pub fn free<T>(ptr: *mut T) -> Result<(), DeallocationError> {
    if ptr.is_null() {
        return Err(DeallocationError::NullPtr);
    }

    if 0 != ptr as usize % HEADER_SIZE {
        return Err(DeallocationError::ImproperAlignment);
    }

    #[allow(clippy::cast_ptr_alignment)]
    let header_ptr = unsafe { ptr.cast::<u8>().sub(HEADER_SIZE).cast::<AllocationHeader>() };

    if !header_ptr.is_aligned() {
        return Err(DeallocationError::ImproperAlignment);
    }

    let header = unsafe { &mut *header_ptr };

    if header.marker == MARKER_FREE {
        return Err(DeallocationError::DoubleFree);
    } else if header.marker != MARKER_USED {
        return Err(DeallocationError::InvalidAllocation);
    }

    let layout = Layout::from_size_align(header.size, HEADER_SIZE)?;

    header.marker = MARKER_FREE;

    unsafe { std::alloc::dealloc(header_ptr.cast(), layout) };

    Ok(())
}

/// Reallocates memory allocated by [`alloc`].
/// # Errors
/// - `AllocationError` if `alloc()` fails
/// - `DeallocationError` if `free(ptr)` fails
/// - `FreeFailedTwice` if `free(ptr)` fails and freeing the newly allocate pointer also fails
/// - `ImproperAlignment` if `ptr` is not properly aligned
/// - `InvalidPointer` if `ptr` was not allocated by [`alloc`] or is invalid
/// - `UseAfterFree` if you try to `realloc` a freed pointer
pub fn relloc(ptr: *mut u8, new_size: usize) -> Result<*mut u8, ReallocationError> {
    if 0 == new_size {
        if !ptr.is_null() {
            free(ptr)?;
        }

        return Ok(std::ptr::null_mut());
    }

    if ptr.is_null() {
        return Ok(alloc(new_size)?);
    }

    if 0 != ptr as usize % HEADER_SIZE {
        return Err(ReallocationError::ImproperAlignment);
    }

    #[allow(clippy::cast_ptr_alignment)]
    let header_ptr = unsafe { ptr.sub(HEADER_SIZE) }.cast::<AllocationHeader>();

    if !header_ptr.is_aligned() {
        return Err(ReallocationError::ImproperAlignment);
    }

    let header = unsafe { &*header_ptr };

    if header.marker == MARKER_FREE {
        return Err(ReallocationError::UseAfterFree);
    } else if header.marker != MARKER_USED {
        return Err(ReallocationError::InvalidPointer);
    }

    let new_ptr = alloc(new_size)?;

    unsafe {
        std::ptr::copy_nonoverlapping::<u8>(ptr, new_ptr, header.size.min(new_size));
    }

    let free_result = free(ptr);

    match free_result {
        Ok(()) => Ok(new_ptr),
        Err(err) => match free(new_ptr) {
            Ok(()) => Err(err)?,
            Err(err2) => Err(ReallocationError::FreeFailedTwice(err, err2)),
        },
    }
}
