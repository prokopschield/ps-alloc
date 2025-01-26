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
pub fn alloc(size: usize) -> Result<*mut u8, AllocationError> {
    use AllocationError::*;

    let size = size
        .div_ceil(ALIGNMENT)
        .checked_add(1)
        .ok_or(ArithmeticError)?
        .checked_mul(ALIGNMENT)
        .ok_or(ArithmeticError)?;

    let layout = Layout::from_size_align(size, ALIGNMENT)?;

    let ptr = unsafe { std::alloc::alloc(layout) };

    if ptr.is_null() {
        Err(OutOfMemory)?
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
pub fn free<T>(ptr: *mut T) -> Result<(), DeallocationError> {
    use DeallocationError::*;

    if ptr.is_null() {
        Err(NullPtr)?
    }

    let ptr = ptr.cast::<Allocation>();

    if !ptr.is_aligned() {
        Err(ImproperAlignment)?
    }

    let ptr = unsafe { ptr.sub(1) };
    let allocation = unsafe { &mut *ptr };

    if allocation.marker == MARKER_FREE {
        Err(DoubleFree)?
    } else if allocation.marker != MARKER_USED {
        Err(InvalidAllocation)?
    }

    let layout = Layout::from_size_align(allocation.size, ALIGNMENT)?;

    allocation.marker = MARKER_FREE;

    unsafe { std::alloc::dealloc(ptr.cast(), layout) }

    Ok(())
}
