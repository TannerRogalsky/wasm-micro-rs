# Minimalist WASM from Rust

```bash
rustc \
    --target=wasm32-unknown-unknown \   # wasm triple
    --emit llvm-ir \                    # we will use llvm tools for the further steps
    --crate-type staticlib \            # clump everything together
    -O \                                # release mode strips panics which cause problems
    add.rs
llc \
    -march=wasm32 \                     # target wasm
    -filetype=obj \                     # output an object file
    add.ll
wasm-objdump -x add.o                   # for debugging
wasm-ld \
    --no-entry \                        # no entry function
    --export-all \                      # export all symbols
    -zstack-size=$[8 * 1024 * 1024] \   # optionally set wasm memory size (ex: 8MiB)
    -o add.wasm \
    add.o
wasm2wat -o add.wast add.wasm           # for debugging


rustc.exe --target=wasm32-unknown-unknown --emit llvm-ir --crate-type staticlib -O -o pkg\add.ll src\add.rs
llc -march=wasm32 -filetype=obj add.ll && wasm-ld --no-entry --export-all -o add.wasm add.o && wasm2wat -o add.wast add.wasm
```
