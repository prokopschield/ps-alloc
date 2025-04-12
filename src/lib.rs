mod error;

use std::alloc::Layout;

pub use error::*;

pub const ALIGNMENT: usize = std::mem::size_of::<Allocation>();
pub const MARKER_FREE: [u8; 8] = *b"Fr33Mmry";
pub const MARKER_USED: [u8; 8] = *b"U53dMmry";

#[repr(align(16))]
struct Allocation {
    marker: [u8; 8],
    size: usize,
}

/// - A reasonably safe implementation of `alloc`.
/// - Memory allocated by this function must be freed by this crate's `free`.
/// - Caller guarantees `free` is called before the returned pointer goes out of scope.
/// # Errors
/// - `Err(ArithmeticError)` is returned on integer overflow, which shouldn't happen.
/// - `Err(LayoutError)` is returned if `sizeof(([u8; 8], usize))` isn't a power of 2.
/// - `Err(OutOfMemory)` is returned if `alloc()` returned a `nullptr`.
#[allow(clippy::cast_ptr_alignment)]
pub fn alloc(size: usize) -> Result<*mut u8, AllocationError> {
    let size = size
        .div_ceil(ALIGNMENT)
        .checked_add(1)
        .ok_or(AllocationError::ArithmeticError)?
        .checked_mul(ALIGNMENT)
        .ok_or(AllocationError::ArithmeticError)?;

    let layout = Layout::from_size_align(size, ALIGNMENT)?;

    let ptr = unsafe { std::alloc::alloc(layout) };

    if ptr.is_null() {
        return Err(AllocationError::OutOfMemory);
    }

    if 0 != (ptr as usize % ALIGNMENT) {
        unsafe { std::alloc::dealloc(ptr, layout) };

        return Err(AllocationError::ImproperAlignment);
    }

    let allocation = unsafe { &mut *(ptr.cast::<Allocation>()) };

    allocation.marker = MARKER_USED;
    allocation.size = size;

    let ptr = unsafe { ptr.add(ALIGNMENT) };

    Ok(ptr)
}

/// - A reasonably safe implementation of `free`.
/// - This function will free a pointer allocated by `alloc`.
/// - Caller guarantees that the provided pointer was allocated by this crate's `alloc` function.
/// - Providing `NULL` is safe and will return `Err(DeallocationError::NullPtr)`.
/// - Providing any other pointer is undefined behaviour.
/// # Errors
/// - Returns `Err(DeallocationError)` if a safety check fails.
pub fn free<T>(ptr: *mut T) -> Result<(), DeallocationError> {
    if ptr.is_null() {
        return Err(DeallocationError::NullPtr);
    }

    let ptr = ptr.cast::<Allocation>();

    if !ptr.is_aligned() {
        return Err(DeallocationError::ImproperAlignment);
    }

    let ptr = unsafe { ptr.sub(1) };
    let allocation = unsafe { &mut *ptr };

    if allocation.marker == MARKER_FREE {
        return Err(DeallocationError::DoubleFree);
    } else if allocation.marker != MARKER_USED {
        return Err(DeallocationError::InvalidAllocation);
    }

    let layout = Layout::from_size_align(allocation.size, ALIGNMENT)?;

    allocation.marker = MARKER_FREE;

    unsafe { std::alloc::dealloc(ptr.cast(), layout) }

    Ok(())
}
