use std::os::raw::{c_ulong, c_void};

#[cfg(target_arch = "wasm32")]
use std::alloc::GlobalAlloc;
#[cfg(target_arch = "wasm32")]
use wee_alloc::WeeAlloc;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: WeeAlloc = WeeAlloc::INIT;

/// Cross-platform malloc function that works for both WASM and native targets
pub unsafe extern "C" fn cross_platform_malloc(size: usize) -> *mut c_void {
    #[cfg(target_arch = "wasm32")]
    {
        // Use wee_alloc for WASM
        let ptr = ALLOC.alloc(std::alloc::Layout::from_size_align_unchecked(size, 1));
        ptr as *mut c_void
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Use libc for native targets
        libc::malloc(size)
    }
}

/// Cross-platform free function that works for both WASM and native targets
pub unsafe extern "C" fn cross_platform_free(ptr: *mut c_void) {
    #[cfg(target_arch = "wasm32")]
    {
        // Use wee_alloc for WASM
        if !ptr.is_null() {
            ALLOC.dealloc(
                ptr as *mut u8,
                std::alloc::Layout::from_size_align_unchecked(1, 1),
            );
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Use libc for native targets
        libc::free(ptr);
    }
}

/// Type-safe wrapper for size conversions
pub fn usize_to_c_ulong(size: usize) -> c_ulong {
    #[cfg(target_arch = "wasm32")]
    {
        // On WASM32, usize is 32-bit, so we need to ensure it fits in c_ulong
        if size > c_ulong::MAX as usize {
            panic!("Size too large for WASM32 target");
        }
        size as c_ulong
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        // On 64-bit targets, this should be safe
        size as c_ulong
    }
}
