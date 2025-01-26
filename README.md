# ps-alloc - a reasonably safe allocator

This crate provides two methods - `alloc` and `free`.

While this crate does implement several safety precautions, you still shouldn't call `free` on stuff willy-nilly, because that _is_ undefined behaviour.

`free` is **NOT** guaranteed to fail when provided anything other than a valid pointer allocated by `alloc`.

**Do not** call `free` any pointers not allocated by `alloc`.

Both `alloc` and `free` return `Result`s. `alloc` returning an `Err` does not signify a problem. `free` returning any error besides `NullPtr` means your program is alredy in an undefined state and you should consider aborting it.
