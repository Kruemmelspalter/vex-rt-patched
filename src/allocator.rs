use core::alloc::Layout;
use newlib_alloc::Alloc;

#[global_allocator]
static ALLOCATOR: Alloc = Alloc;

#[alloc_error_handler]
fn handle(layout: Layout) -> ! {
    panic!("memory allocation failed: {:#?}", layout);
}
