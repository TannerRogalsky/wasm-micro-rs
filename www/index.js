import * as wasm from "wasm-micro-rs";
{
    const addResult = wasm.add(1, 3);
    console.log(addResult);

    const sumResult = wasm.sum([1, 2, 3]);
    console.log(sumResult);
}

import * as lowwasm from '../pkg/add.wasm';
{

    const addResult = lowwasm.add(1, 3);
    console.log(addResult);

    const jsArray = [1, 2, 3];
    const cArrayPointer = lowwasm.malloc(jsArray.length * 4);
    const cArray = new Uint32Array(
        lowwasm.memory.buffer,
        cArrayPointer,
        jsArray.length
    );
    cArray.set(jsArray);
    let sumResult = lowwasm.sum(cArrayPointer, cArray.length);
    console.log(sumResult);
}
