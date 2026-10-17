//! Backend/runtime regressions using original declarations and a native C driver.
use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, TargetOs, VirtualFs};
use jaic_llvm::{OptLevel, emit_object};
use std::{path::PathBuf, process::Command, rc::Rc};

const SOURCE: &str = r#"
atomic_load :: (pointer: *$T) -> T #intrinsic;
atomic_store :: (pointer: *$T, new: T) #intrinsic;
atomic_exchange :: (pointer: *$T, new: T) -> T #intrinsic;
compare_and_swap :: (pointer: *$T, old: T, new: T) -> (bool, T) #intrinsic;
atomic_counter: u64;
exchange_counter: u64;
publication_data: u64;
publication_ready: u64;
#program_export "atomic_address"
address :: () -> *u64 #c_call { return *atomic_counter; }
#program_export "exchange_address"
exchange_address :: () -> *u64 #c_call { return *exchange_counter; }
#program_export "publication_data_address"
publication_data_address :: () -> *u64 #c_call { return *publication_data; }
#program_export "publication_ready_address"
publication_ready_address :: () -> *u64 #c_call { return *publication_ready; }
#program_export "atomic_replace"
replace :: (p: *u64, value: u64) -> u64 #c_call { return atomic_exchange(p, value); }
#program_export "atomic_publish"
publish :: (data: *u64, ready: *u64, value: u64) #c_call {
    while atomic_load(ready) != 0 {}
    data.* = value;
    atomic_store(ready, 1);
}
#program_export "atomic_observe"
observe :: (data: *u64, ready: *u64) -> u64 #c_call {
    while atomic_load(ready) == 0 {}
    result := data.*;
    atomic_store(ready, 0);
    return result;
}
#program_export "atomic_step"
step :: (p: *u64) -> u64 #c_call {
    while true {
        old := atomic_load(p);
        ok, observed := compare_and_swap(p, old, old+1);
        if ok return observed;
    }
}
#program_export "atomic_check"
check :: () -> s32 #c_call {
    a: u8; b: s16; c: u32; d: u64; f: float32; g: float64; flag: bool;
    atomic_store(*a, 255); atomic_store(*b, -7);
    atomic_store(*c, 0xf1234567); atomic_store(*d, 0xfedcba9876543210);
    atomic_store(*f, 0h80000000); atomic_store(*g, 0h7ff8000000001234);
    atomic_store(*flag, true);
    ok_nan, nan_old := compare_and_swap(*g, 0h7ff8000000001234, 0h7ff8000000005678);
    if !ok_nan || cast,force(u64) nan_old != 0x7ff8000000001234 return 8;
    atomic_store(*g, 0h7ff8000000001234);
    if atomic_load(*a) != 255 || atomic_exchange(*b, 9) != -7 return 1;
    if atomic_load(*c) != 0xf1234567 || atomic_exchange(*d, 17) != 0xfedcba9876543210 return 2;
    if cast,force(u32) atomic_load(*f) != 0x80000000 return 3;
    if cast,force(u64) atomic_exchange(*g, 1) != 0x7ff8000000001234 return 4;
    if !atomic_exchange(*flag, false) || atomic_load(*flag) return 5;
    p: *u32; atomic_store(*p, *c);
    if atomic_load(*p) != *c || atomic_exchange(*p, null) != *c return 6;
    ok, previous := compare_and_swap(*d, 17, 23);
    if !ok || previous != 17 || d != 23 return 7;
    return 0;
}
"#;

fn directory(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("atomic-backend")
        .join(name);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn object(dir: &std::path::Path, wasm: bool, optimized: bool) -> PathBuf {
    let mut options = Options::host();
    options.runtime_support = false;
    if wasm {
        options.os = TargetOs::Wasm;
        options.cpu = TargetCpu::Wasm;
    }
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    if let Err(e) = compiler.compile_sources(&[ProgramSource::String(SOURCE.into())]) {
        panic!("{}", e.render(&compiler.sources));
    }
    let object = dir.join("atomics.o");
    let options = jaic_llvm::Options {
        target: wasm.then(|| "wasm64-unknown-unknown".into()),
        opt_level: if optimized {
            OptLevel::O2
        } else {
            OptLevel::O0
        },
        emit_ir: Some(dir.join("atomics.ll")),
        ..Default::default()
    };
    emit_object(&compiler.program, &options, &object).unwrap();
    let ir = std::fs::read_to_string(dir.join("atomics.ll")).unwrap();
    for text in [
        "load atomic",
        "store atomic",
        "atomicrmw xchg",
        "cmpxchg",
        "seq_cst",
    ] {
        assert!(ir.contains(text), "missing {text}");
    }
    object
}

fn run(command: &mut Command) -> std::process::Output {
    let output = command
        .output()
        .expect("required test tool must be installed");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
#[cfg(unix)]
fn native_scalars_alignment_and_real_thread_contention() {
    for optimized in [false, true] {
        let dir = directory(if optimized {
            "native-o2"
        } else {
            "native-o0"
        });
        let object = object(&dir, false, optimized);
        let source = dir.join("driver.c");
        std::fs::write(&source, r#"
#include <pthread.h>
#include <stdint.h>
#include <stdatomic.h>
#include <unistd.h>
extern uint64_t atomic_step(uint64_t *);
extern int atomic_check(void);
extern uint64_t atomic_replace(uint64_t *, uint64_t);
extern void atomic_publish(uint64_t *, uint64_t *, uint64_t);
extern uint64_t atomic_observe(uint64_t *, uint64_t *);
static _Atomic uint64_t count;
static _Atomic uint64_t exchanges;
static uint64_t sums[4], exchange_sums[4];
static _Alignas(8) uint64_t data;
static _Atomic uint64_t ready;
static int publication_failed;
static void *worker(void *arg) {
    uintptr_t index=(uintptr_t)arg;
    for (int i=0;i<30000;i++) {
        sums[index]+=atomic_step((uint64_t*)&count);
        exchange_sums[index]+=atomic_replace((uint64_t*)&exchanges,index*30000+i+1);
    }
    return 0;
}
static void *producer(void *arg) {
    for(uint64_t i=1;i<=30000;i++) atomic_publish(&data,(uint64_t*)&ready,i);
    return 0;
}
static void *consumer(void *arg) {
    for(uint64_t i=1;i<=30000;i++) if(atomic_observe(&data,(uint64_t*)&ready)!=i) publication_failed=1;
    return 0;
}
int main(int argc, char **argv) {
    if (argc>1) { _Alignas(8) unsigned char bytes[24]={0}; atomic_step((uint64_t*)(bytes+1)); return 98; }
    alarm(15);
    if (atomic_check()) return 1;
    pthread_t threads[4];
    for (uintptr_t i=0;i<4;i++) if(pthread_create(&threads[i],0,worker,(void*)i)) return 2;
    for (int i=0;i<4;i++) if(pthread_join(threads[i],0)) return 3;
    uint64_t sum=sums[0]+sums[1]+sums[2]+sums[3];
    if(atomic_load(&count)!=120000 || sum!=UINT64_C(120000)*119999/2) return 4;
    uint64_t exchange_sum=atomic_load(&exchanges);
    for(int i=0;i<4;i++) exchange_sum+=exchange_sums[i];
    if(exchange_sum!=UINT64_C(120000)*120001/2) return 5;
    if(pthread_create(&threads[0],0,producer,0) || pthread_create(&threads[1],0,consumer,0)) return 6;
    if(pthread_join(threads[0],0) || pthread_join(threads[1],0)) return 7;
    return publication_failed ? 8:0;
}
"#).unwrap();
        let executable = dir.join("driver");
        run(Command::new("clang")
            .args(["-std=c11", "-pthread"])
            .arg(&source)
            .arg(object)
            .arg("-o")
            .arg(&executable));
        run(&mut Command::new(&executable));
        let bad = Command::new(&executable)
            .arg("misaligned")
            .output()
            .unwrap();
        assert!(!bad.status.success() && bad.status.code() != Some(98));
    }
}

#[test]
fn wasm64_scalar_operations_execute() {
    let dir = directory("wasm64");
    let object = object(&dir, true, true);
    let module = dir.join("atomics.wasm");
    run(Command::new("wasm-ld")
        .args([
            "-mwasm64",
            "--no-entry",
            "--export=atomic_check",
            "--export=atomic_step",
            "--allow-undefined",
        ])
        .arg(object)
        .arg("-o")
        .arg(&module));
    let script = r#"const fs=require('fs'); const m=new WebAssembly.Module(fs.readFileSync(process.argv[1]));
const env={}; for(const i of WebAssembly.Module.imports(m)){if(i.kind!=='function')throw Error('unsupported import');env[i.name]=()=>{throw Error('unexpected runtime failure: '+i.name);};}
const instance=new WebAssembly.Instance(m,{env}); if(instance.exports.atomic_check()!==0)throw Error('atomic scalar mismatch');"#;
    run(Command::new("node").args(["-e", script]).arg(module));
}

#[test]
fn wasm_cannot_silently_disable_required_atomics() {
    let dir = directory("wasm64-disabled");
    let mut options = Options::host();
    options.runtime_support = false;
    options.os = TargetOs::Wasm;
    options.cpu = TargetCpu::Wasm;
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    compiler
        .compile_sources(&[ProgramSource::String(SOURCE.into())])
        .unwrap();
    let options = jaic_llvm::Options {
        target: Some("wasm64-unknown-unknown".into()),
        features: Some("-atomics".into()),
        ..Default::default()
    };
    let error = emit_object(&compiler.program, &options, &dir.join("disabled.o"))
        .expect_err("explicitly disabled required atomics must fail");
    assert!(error.contains("atomics target feature"), "{error}");
}

#[test]
fn wasm64_shared_memory_contention() {
    let dir = directory("wasm64-shared");
    let object = object(&dir, true, true);
    let module = dir.join("primary.wasm");
    let imported = dir.join("worker.wasm");
    for (path, import_memory) in [(&module, false), (&imported, true)] {
        let mut linker = Command::new("wasm-ld");
        linker.args([
            "-mwasm64",
            "--no-entry",
            "--export=atomic_step",
            "--export=atomic_address",
            "--export=exchange_address",
            "--export=atomic_replace",
            "--export=atomic_publish",
            "--export=atomic_observe",
            "--export=publication_data_address",
            "--export=publication_ready_address",
            "--export-memory",
            "--allow-undefined",
            "--shared-memory",
            "--max-memory=16777216",
        ]);
        if import_memory {
            linker.arg("--import-memory");
        }
        run(linker.arg(&object).arg("-o").arg(path));
    }
    let script = r#"
const fs=require('fs'),{Worker}=require('worker_threads');
function env(m,memory){const e={memory};for(const i of WebAssembly.Module.imports(m))if(i.kind==='function')e[i.name]=()=>{throw Error('unexpected runtime failure '+i.name);};return e;}
const primary=new WebAssembly.Module(fs.readFileSync(process.argv[1]));
const first=new WebAssembly.Instance(primary,{env:env(primary)});
const memory=first.exports.memory, pointer=first.exports.atomic_address(), exchangePointer=first.exports.exchange_address();
const module=new WebAssembly.Module(fs.readFileSync(process.argv[2]));
const workerSource=`const {parentPort,workerData:d}=require('worker_threads');
const e={memory:d.memory};for(const i of WebAssembly.Module.imports(d.module))if(i.kind==='function')e[i.name]=()=>{throw Error('unexpected runtime failure');};
const x=new WebAssembly.Instance(d.module,{env:e});parentPort.postMessage('ready');
parentPort.once('message',()=>{let sum=0n,exchanges=0n;for(let i=0;i<30000;i++){sum+=x.exports.atomic_step(d.pointer);exchanges+=x.exports.atomic_replace(d.exchangePointer,BigInt(d.index*30000+i+1));}parentPort.postMessage([sum,exchanges]);});`;
const workers=[],ready=[],done=[];
for(let i=0;i<4;i++){
 const w=new Worker(workerSource,{eval:true,workerData:{memory,module,pointer,exchangePointer,index:i}});workers.push(w);
 ready.push(new Promise((resolve,reject)=>{w.once('message',resolve);w.once('error',reject);}));
 done.push(new Promise((resolve,reject)=>{w.on('message',m=>{if(Array.isArray(m))resolve(m);});w.once('error',reject);}));
}
setTimeout(()=>{throw Error('contention timeout');},15000).unref();
(async()=>{await Promise.all(ready);for(const w of workers)w.postMessage('go');const sums=await Promise.all(done);
const count=Atomics.load(new BigUint64Array(memory.buffer),Number(pointer/8n));
if(count!==120000n || sums.reduce((a,b)=>a+b[0],0n)!==120000n*119999n/2n)throw Error('lost atomic updates '+count);
const last=Atomics.load(new BigUint64Array(memory.buffer),Number(exchangePointer/8n));
if(sums.reduce((a,b)=>a+b[1],last)!==120000n*120001n/2n)throw Error('lost exchange values');
for(const w of workers)await w.terminate();
const data=first.exports.publication_data_address(),flag=first.exports.publication_ready_address();
const publicationSource=`const {parentPort,workerData:d}=require('worker_threads');
const e={memory:d.memory};for(const i of WebAssembly.Module.imports(d.module))if(i.kind==='function')e[i.name]=()=>{throw Error('unexpected runtime failure');};
const x=new WebAssembly.Instance(d.module,{env:e});parentPort.postMessage('ready');
parentPort.once('message',()=>{for(let i=1n;i<=30000n;i++){if(d.producer)x.exports.atomic_publish(d.data,d.flag,i);else if(x.exports.atomic_observe(d.data,d.flag)!==i)throw Error('publication order mismatch');}parentPort.postMessage('done');});`;
const pair=[],pairReady=[],pairDone=[];
for(const producer of [false,true]){
 const w=new Worker(publicationSource,{eval:true,workerData:{memory,module,data,flag,producer}});pair.push(w);
 pairReady.push(new Promise((resolve,reject)=>{w.once('message',resolve);w.once('error',reject);}));
 pairDone.push(new Promise((resolve,reject)=>{w.on('message',m=>{if(m==='done')resolve();});w.once('error',reject);}));
}
await Promise.all(pairReady);for(const w of pair)w.postMessage('go');await Promise.all(pairDone);for(const w of pair)await w.terminate();
})().catch(e=>{console.error(e);process.exit(1);});
"#;
    run(Command::new("node")
        .args(["-e", script])
        .arg(module)
        .arg(imported));
}
