use core::alloc::{GlobalAlloc, Layout};

use libc::{free, memalign};

struct Alloc;

unsafe impl GlobalAlloc for Alloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        rtos_suspend_all();
        let mem = memalign(layout.align(), layout.size()) as *mut _;
        rtos_resume_all();
        mem
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        rtos_suspend_all();
        free(ptr as *mut _);
        rtos_resume_all();
    }
}

extern "C" {
    fn rtos_suspend_all();
    fn rtos_resume_all();
}

#[global_allocator]
static ALLOCATOR: Alloc = Alloc;

#[alloc_error_handler]
fn handle(layout: Layout) -> ! {
    panic!("memory allocation failed: {:#?}", layout);
}
