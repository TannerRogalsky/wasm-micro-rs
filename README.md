# Minimalist WASM from Rust

Inspired by Surma's excellent post about "[Compiling C to WebAssembly without Emscripten](https://dassur.ma/things/c-to-webassembly/)", I wanted to explore creating a similarly stripped down environment in Rust.

To summarize: let's use Rust to create some WebAssembly without any of the batteries that Emscripten provides.

## No Additional Cargo

Starting simply, we'll create a function that adds two integers together.

```rust
#![no_std]

#[no_mangle]
pub extern fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

Hopefully the function signature and body are straightforward. Some of the attributes are non-standard, though.

The `#![no_std]` attribute tells the Rust compiler not to include Rust's standard library. This isn't going to impact our limited example's size but it is good practise!

`#[no_mangle]` will disable Rust's name mangling for a function and make it easy to link to.

The `extern` in the function signature indicates that we want it available from whatever is using the library. It also makes the function adhere to the C calling convention.

## Initial Compilation

```bash
rustc \
    --target=wasm32-unknown-unknown \   # wasm triple
    --emit llvm-ir \                    # we will use llvm tools for the further steps
    --crate-type staticlib \            # clump everything together
    -O \                                # release mode strips panics which cause problems
    -o add.ll \                         # output file
    add.rs
```

But when we try to compile our code we get ``error: `#[panic_handler]` function required, but not found``. Because we aren't including std's panic handler we have to provide our own, even though we're building release code and it'll be stripped out. We'll add a tiny one at the bottom of our file.

```rust
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
```

Rerun the previous command and you should get a file called add.ll which has a human-readable content. When we told Rust to emit `llvm-ir` what that stands for is "LLVM Intermediate Representation". This is useful to us because LLVM is the compiler used by the Emscripten toolchain and we can turn this Intermediate Representation in WebAssembly.

```llvm-ir
; ModuleID = 'add.3a1fbbbh-cgu.0'
source_filename = "add.3a1fbbbh-cgu.0"
target datalayout = "e-m:e-p:32:32-i64:64-n32:64-S128"
target triple = "wasm32-unknown-unknown"

; Function Attrs: norecurse nounwind readnone
define i32 @add(i32 %a, i32 %b) unnamed_addr #0 {
start:
  %0 = add i32 %b, %a
  ret i32 %0
}

attributes #0 = { norecurse nounwind readnone "target-cpu"="generic" }
```

Very nice! We can definitely see the similarity between our code and this IR.

## Linking And Debugging

The following steps are very similar to Surma's article starting from the "Turning LLVM IR into object files" section so I will brush over them. Please refer back to that if you want a more detailed explanation.

```bash
llc \
    -march=wasm32 \                     # target wasm
    -filetype=obj \                     # output an object file
    -O3                                 # maximum optimization level
    add.ll
wasm-objdump -x add.o                   # for debugging
wasm-ld \
    --no-entry \                        # no entry function
    --export=add \                      # just export add
    -zstack-size=$[8 * 1024 * 1024] \   # optionally set wasm memory size (ex: 8MiB)
    -o add.wasm \
    add.o
wasm-opt \
    --strip-producers \                 # https://github.com/WebAssembly/tool-conventions/blob/master/ProducersSection.md
    -Oz \                               # smol
    -o ./pkg/add.wasm \
    ./pkg/add.wasm
wasm2wat -o add.wast add.wasm           # for debugging
```

Executing these commands will result in a WebAssembly module that you can run and call the exported `add` function.

```js
wasm.add(1, 3); // = 4
```

### Dynamic Memory

Just because we're not using std doesn't mean we can't allocate memory.

```rust
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
```

The world's worst allocator.

```rust
#[no_mangle]
pub extern fn sum(slice: &[i32]) -> i32 {
    slice.iter().sum()
}
```

Calling this function means we first have to allocate memory on the WASM heap and then copy data into that memory. Only then can we execute our sum function.

```js
const jsArray = [1, 2, 3];                              // data
const cArrayPointer = wasm.malloc(jsArray.length * 4);  // heap allocation
const cArray = new Uint32Array(                         // memory view
    wasm.memory.buffer,
    cArrayPointer,
    jsArray.length
);
cArray.set(jsArray);                                    // memcpy
wasm.sum(cArrayPointer, cArray.length);                 // = 6
```

## Using Cargo

It's interesting to use the basic tools to build our binary but one of the appeals of Rust is that it has tooling built around it that makes this stuff easier for us. Using Cargo, wasm-pack and wasm-bindgen add some additional weight but maybe not as much as you'd expect with some tuning.

```rust
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
```

With Rust 1.36.0 and LLVM 8.0 and wasm-pack#439e5231 (for wasm-opt integration), this results in a WASM file that is 216 bytes whereas our completely handrolled one clocks in at 208 bytes.

Most of this additional weight comes from the additional functionality provided by the GlobalAlloc trait. Passing the `Layout` information isn't completely free.

A difference whose origin I haven't had the chance to track down is the wasm-pack WASM containing a single `data` entry as opposed to our handrolled one's multiple globals and exports.

## Tools
- rustc 1.36.0 (a53f9df32 2019-07-03)
- llvm 8.0.0
- wasm-pack#439e5231
- wabt 1.0.11 (for wasm-objdump & wasm2wat)
- binaryen stable 87 (for wasm-opt)

```
rustc.exe --target=wasm32-unknown-unknown --emit llvm-ir --crate-type staticlib -O -o pkg\add.ll src\add.rs
llc -march=wasm32 -filetype=obj add.ll && wasm-ld --no-entry --export=add --export=sum --export=malloc -o add.wasm add.o && wasm2wat -o add.wast add.wasm
```
