use core::alloc::{GlobalAlloc, Layout};

use libc::{free, memalign};

struct Alloc;

unsafe impl GlobalAlloc for Alloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        __malloc_lock();
        let mem = memalign(layout.align(), layout.size()) as *mut _;
        __malloc_unlock();
        mem
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        __malloc_lock();
        free(ptr as *mut _);
        __malloc_unlock();
    }
}

extern "C" {
    fn __malloc_lock();
    fn __malloc_unlock();
}

#[global_allocator]
static ALLOCATOR: Alloc = Alloc;

#[alloc_error_handler]
fn handle(layout: Layout) -> ! {
    panic!("memory allocation failed: {:#?}", layout);
}
