#![no_std]
extern crate wasm_bindgen;

use wasm_bindgen::prelude::wasm_bindgen;
use core::alloc::{GlobalAlloc, Layout};

extern {
    static mut __heap_base: u8;
}
static mut BUMP_POINTER : isize = 0;

struct BadAllocator;

unsafe impl GlobalAlloc for BadAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let r : *mut u8 = (&mut __heap_base as *mut u8).offset(BUMP_POINTER);
        BUMP_POINTER += layout.size() as isize;
        r
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // ohno.jpg
    }
}
#[global_allocator]
static ALLOC: BadAllocator = BadAllocator;

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[wasm_bindgen]
pub fn sum(slice : &[i32]) -> i32 {
    slice.iter().sum()
}
