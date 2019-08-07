#!/usr/bin/env bash

rustc \
    --target=wasm32-unknown-unknown \
    --emit llvm-ir \
    --crate-type staticlib \
    -C opt-level=s \
    -o ./pkg/add.ll \
    ./src/add.rs

llc \
    -march=wasm32 \
    -filetype=obj \
    -O3 \
    ./pkg/add.ll

wasm-objdump -s -d -x ./pkg/add.o > ./pkg/add.wasmdeb

wasm-ld \
    --no-entry \
    --export=add --export=sum --export=malloc \
    -zstack-size=$[8 * 1024 * 1024] \
    -o ./pkg/add.wasm \
    ./pkg/add.o

wasm-opt \
    --strip-producers \
    -Oz \
    -o ./pkg/add.wasm \
    ./pkg/add.wasm

wasm2wat -o ./pkg/add.wast ./pkg/add.wasm
