#![no_std]

#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
pub extern "C" fn sum(slice: &[i32]) -> i32 {
    slice.iter().sum()
}

extern "C" {
    static mut __heap_base: u8;
}
static mut BUMP_POINTER: isize = 0;

#[no_mangle]
unsafe extern "C" fn malloc(n: isize) -> *mut u8 {
    let r: *mut u8 = (&mut __heap_base as *mut u8).offset(BUMP_POINTER);
    BUMP_POINTER += n;
    r
}

#[no_mangle]
unsafe extern "C" fn free(_p: *const u8) {
    // ohno.jpg
}

// IGNORE ME
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
