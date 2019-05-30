#![no_std]

#[no_mangle]
pub extern fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
pub extern fn sum(slice: &[i32]) -> i32 {
    slice.iter().sum()
}

extern {
    static __heap_base: usize;
}
static mut BUMP_POINTER : isize = 0;

#[no_mangle]
unsafe extern fn malloc(n: isize) -> *const usize {
    let r : *const usize = (&__heap_base as *const usize).offset(BUMP_POINTER);
    BUMP_POINTER += n;
    r
}

#[no_mangle]
unsafe extern fn free(_p: *const usize) {
    // ohno.jpg
}

// IGNORE ME
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
