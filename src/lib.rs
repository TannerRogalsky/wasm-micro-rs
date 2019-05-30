#![no_std]

extern crate wee_alloc;
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

extern crate wasm_bindgen;

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[wasm_bindgen]
pub fn sum(slice : &[i32]) -> i32 {
    slice.iter().sum()
}
