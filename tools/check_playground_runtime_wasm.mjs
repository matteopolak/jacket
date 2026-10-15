// Real wasm32 compiler execution of the private sandbox scheduler/filesystem provider.
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createEngine } from "../crates/jai-wasm/js/engine.mjs";

const binary=process.argv[2];
if(!binary)throw new Error("usage: node tools/check_playground_runtime_wasm.mjs <jai_wasm.wasm>");
const engine=await createEngine(await readFile(binary));
for(const name of ["cooperative-runtime","sandbox-files"]) {
    const source=await readFile(new URL(`../crates/jai-wasm/tests/fixtures/${name}.jai`,import.meta.url),"utf8");
    for(let i=0;i<3;++i) {
        const result=engine.play({"main.jai":source},"main.jai");
        assert.deepEqual(result.diagnostics,[],result.rendered);
        assert.equal(result.exitCode,0,result.rendered);
        assert.equal(result.stdout,"ok\n");
    }
}
const isolated=engine.play({"main.jai":`fopen :: (path:*u8,mode:*u8)->*void #foreign; main :: ()->int {if fopen("/tmp/x.txt\\0".data,"rb\\0".data)!=null return 1;return 0;}`},"main.jai");
assert.deepEqual(isolated.diagnostics,[],isolated.rendered);
assert.equal(isolated.exitCode,0);
const guard=engine.play({"main.jai":'#import "Thread"; main :: () {}'},"main.jai");
assert.equal(guard.exitCode,null);
assert(guard.diagnostics.some(d=>d.code==="static-assert") && guard.rendered.includes("OS"),guard.rendered);
const invalidJoin=engine.play({"main.jai":"pthread_join :: (thread:u64,result:**void)->s32 #foreign; main :: ()->int {if pthread_join(0,null)==0 return 1;return 0;}"},"main.jai");
assert.deepEqual(invalidJoin.diagnostics,[],invalidJoin.rendered);
assert.equal(invalidJoin.exitCode,0);
const deadlock=engine.play({"main.jai":`
    pthread_create :: (thread:*u64,attributes:*void,entry:*void,data:*void)->s32 #foreign;
    pthread_join :: (thread:u64,result:**void)->s32 #foreign;
    pthread_mutex_init :: (token:*void,attributes:*void)->s32 #foreign;
    pthread_mutex_lock :: (token:*void)->s32 #foreign;
    mutex:u64;
    worker :: (argument:*void)->*void #c_call {pthread_mutex_lock(*mutex);return argument;}
    main :: ()->int {
        if pthread_mutex_init(*mutex,null)!=0 return 1;
        if pthread_mutex_lock(*mutex)!=0 return 2;
        thread:u64;
        if pthread_create(*thread,null,cast(*void)worker,null)!=0 return 3;
        return pthread_join(thread,null);
    }
`},"main.jai");
assert.equal(deadlock.exitCode,null);
assert(deadlock.diagnostics.some(d=>d.message.includes("deadlock")),deadlock.rendered);
console.log("PASS: actual wasm32 cooperative workers/barrier/contention/joins, in-memory files/source reads/isolation, invalid handles and public Thread rejection");
