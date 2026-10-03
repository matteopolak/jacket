use super::*;
use crate::test_native_tools as native_tools;
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

#[test]
fn reserved_pool_runtime_symbols_cannot_adopt_a_foreign_declaration() {
    let target = target::NativeTarget::new().unwrap();
    let context = Context::create();
    for name in ["jai.pool.get", "jai.pool.reset", "jai.pool.metadata.state"] {
        let module = context.create_module("pool-collision");
        module.add_function(name, context.void_type().fn_type(&[], false), None);
        assert!(matches!(
            prepare(&module, &context, &target),
            Err(Error::RuntimeIntrinsic(_))
        ));
    }
    let module = context.create_module("pool-generated");
    prepare(&module, &context, &target).unwrap();
    prepare(&module, &context, &target).unwrap();
    module.verify().unwrap();
}

#[test]
fn typed_pool_runtime_verifies_selected_pointer_widths() {
    for triple in [
        "i386-unknown-linux-gnu",
        "x86_64-unknown-linux-gnu",
        "aarch64-apple-darwin",
    ] {
        let target = target::NativeTarget::select(&target::TargetOptions {
            selection: target::TargetSelection::Triple(target::Triple::new(triple).unwrap()),
            ..target::TargetOptions::default()
        })
        .unwrap();
        let context = Context::create();
        let module = context.create_module("pool-runtime");
        module.set_data_layout(&target.data.get_data_layout());
        module.set_triple(&target.triple);
        ledger::define(&context, &module, &target).unwrap();
        module.verify().unwrap();
        let calloc = module.get_function("calloc").unwrap();
        assert_eq!(
            calloc.get_type().get_param_types()[0]
                .into_int_type()
                .get_bit_width(),
            context
                .ptr_sized_int_type(&target.data, None)
                .get_bit_width()
        );
    }
}

#[test]
fn native_owned_blocks_reset_reuse_and_free_actual_allocations() {
    run_native(
        r#"
    int main(void) {
        if(get(0,1,0,0,8,0,0,0,0,0) || outstanding) return 10;
        Pool pool={128,0,0,0,16};
        unsigned char *first=get(&pool,1,3,0,8,&pool.capacity,&pool.left,&pool.block,&pool.pos,&pool.alignment);
        unsigned char *second=get(&pool,1,5,0,8,&pool.capacity,&pool.left,&pool.block,&pool.pos,&pool.alignment);
        if ((uintptr_t)first%16 || second-first!=16 || pool.left!=107 || outstanding!=3) return 1;
        *first=42;
        reset(&pool,1,0,0,8,&pool.left,&pool.block,&pool.pos,&pool.alignment);
        if (*first!=42) return 2;
        unsigned char *reused=get(&pool,1,3,0,8,&pool.capacity,&pool.left,&pool.block,&pool.pos,&pool.alignment);
        if (reused!=first) return 3;
        reset(&pool,1,1,0,8,&pool.left,&pool.block,&pool.pos,&pool.alignment);
        if (*first!=0xcc) return 4;
        get(&pool,1,128,0,8,&pool.capacity,&pool.left,&pool.block,&pool.pos,&pool.alignment);
        get(&pool,1,1,0,8,&pool.capacity,&pool.left,&pool.block,&pool.pos,&pool.alignment);
        if (outstanding!=5) return 5;
        release(&pool,1,&pool.left,&pool.block,&pool.pos);
        if (outstanding || pool.left || pool.block || pool.pos) return 6;
        Pool ordinary={0};
        get(&ordinary,0,64,sizeof(void*),8,&ordinary.capacity,&ordinary.left,&ordinary.block,&ordinary.pos,0);
        get(&ordinary,0,64,sizeof(void*),8,&ordinary.capacity,&ordinary.left,&ordinary.block,&ordinary.pos,0);
        if (ordinary.left!=65400 || ordinary.pos!=136 || ordinary.capacity!=65536) return 7;
        release(&ordinary,0,&ordinary.left,&ordinary.block,&ordinary.pos);
        if(outstanding) return 8;
        Pool outer={256,0,0,0,8};
        Pool *inner=get(&outer,1,sizeof(Pool),0,8,&outer.capacity,&outer.left,&outer.block,&outer.pos,&outer.alignment);
        get(inner,1,8,0,8,&inner->capacity,&inner->left,&inner->block,&inner->pos,&inner->alignment);
        release(inner,1,&inner->left,&inner->block,&inner->pos);
        release(&outer,1,&outer.left,&outer.block,&outer.pos);
        return outstanding ? 9 : 0;
    }
    "#,
        false,
    );
}

#[cfg(unix)]
#[test]
fn native_nested_lifetime_and_nominal_ownership_violations_trap() {
    run_native(
        r#"
        #include <signal.h>
        #include <unistd.h>
        #include <sys/wait.h>
        static void invalid(int mode) {
            if(mode==3) { get(0,1,1,0,8,0,0,0,0,0); _exit(99); }
            if(mode==4) { get(0,1,-1,0,8,0,0,0,0,0); _exit(99); }
            Pool outer={256,0,0,0,8};
            Pool *inner=get(&outer,1,sizeof(Pool),0,8,&outer.capacity,&outer.left,&outer.block,&outer.pos,&outer.alignment);
            get(inner,1,8,0,8,&inner->capacity,&inner->left,&inner->block,&inner->pos,&inner->alignment);
            if(mode==0) reset(&outer,1,1,0,8,&outer.left,&outer.block,&outer.pos,&outer.alignment);
            if(mode==1) release(&outer,1,&outer.left,&outer.block,&outer.pos);
            if(mode==2) {
                static char different_nominal_type;
                outer.left=outer.pos=0;outer.block=0;
                jai_fixture_get(&outer,&different_nominal_type,1,8,0,8,&outer.capacity,&outer.left,&outer.block,&outer.pos,&outer.alignment);
            }
            _exit(99);
        }
        int main(void) {
            for(int mode=0;mode<5;mode++) {
                pid_t child=fork();
                if(child<0) return 1;
                if(child==0) invalid(mode);
                int status;
                if(waitpid(child,&status,0)!=child || !WIFSIGNALED(status)) return 2;
                if(WTERMSIG(status)!=SIGILL && WTERMSIG(status)!=SIGTRAP) return 3;
            }
            return 0;
        }
        "#,
        false,
    );
}

#[test]
fn native_shared_pool_allocations_are_serialized_between_threads() {
    run_native(
        r#"
    #include <pthread.h>
    static Pool pool={4096,0,0,0,8};
    static unsigned char *pointers[512];
    static void *worker(void *arg) {
        size_t start=(size_t)arg;
        for(size_t i=start;i<start+256;i++) {
            pointers[i]=get(&pool,1,1,0,8,&pool.capacity,&pool.left,&pool.block,&pool.pos,&pool.alignment);
            *pointers[i]=42;
        }
        return 0;
    }
    int main(void) {
        pthread_t threads[2];
        if(pthread_create(&threads[0],0,worker,(void*)0) || pthread_create(&threads[1],0,worker,(void*)256)) return 1;
        pthread_join(threads[0],0); pthread_join(threads[1],0);
        for(size_t i=0;i<512;i++) {
            if(*pointers[i]!=42) return 2;
            for(size_t j=0;j<i;j++) if(pointers[i]==pointers[j]) return 3;
        }
        release(&pool,1,&pool.left,&pool.block,&pool.pos);
        return outstanding ? 4 : 0;
    }
    "#,
        true,
    );
}

fn run_native(body: &str, threads: bool) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "jai-pool-builder-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&directory).unwrap();
    let context = Context::create();
    let module = context.create_module("pool-fixture");
    let target = target::NativeTarget::new().unwrap();
    module.set_data_layout(&target.data.get_data_layout());
    module.set_triple(&target.triple);
    prepare(&module, &context, &target).unwrap();
    module.verify().unwrap();
    // Independently authored C wrappers count the actual runtime allocations.
    module
        .get_function("calloc")
        .unwrap()
        .as_global_value()
        .set_name("jai_fixture_calloc");
    module
        .get_function("free")
        .unwrap()
        .as_global_value()
        .set_name("jai_fixture_free");
    for (name, fixture) in [
        ("get", "jai_fixture_get"),
        ("reset", "jai_fixture_reset"),
        ("release", "jai_fixture_release"),
    ] {
        let function = module.get_function(&format!("jai.pool.{name}")).unwrap();
        function.set_linkage(inkwell::module::Linkage::External);
        function.as_global_value().set_name(fixture);
    }
    let object = directory.join("pool.o");
    target.write_object(&module, &object).unwrap();
    let source = directory.join("fixture.c");
    fs::write(&source, format!("{HEADER}\n{body}")).unwrap();
    let binary = directory.join("fixture");
    let mut command = native_tools::clang_command();
    command.arg(&source).arg(&object).arg("-o").arg(&binary);
    if threads {
        command.arg("-pthread");
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut child = Command::new(&binary).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert_eq!(status.code(), Some(0));
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            let _ = child.wait();
            panic!("native pool fixture timed out");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    fs::remove_dir_all(directory).unwrap();
}
const HEADER: &str = r#"
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
typedef struct { int64_t capacity,left; void *block; int64_t pos,alignment; } Pool;
static unsigned outstanding;
void *jai_fixture_calloc(size_t count,size_t size) { void *p=calloc(count,size);if(p)outstanding++;return p; }
void jai_fixture_free(void *p) { if(p)outstanding--;free(p); }
static char plain_token,flat_token;
extern void *jai_fixture_get(void*,void*,_Bool,int64_t,int64_t,int64_t,int64_t*,int64_t*,void**,int64_t*,int64_t*);
extern void jai_fixture_reset(void*,void*,_Bool,_Bool,int64_t,int64_t,int64_t*,void**,int64_t*,int64_t*);
extern void jai_fixture_release(void*,void*,_Bool,int64_t*,void**,int64_t*);
#define get(owner,flat,...) jai_fixture_get(owner,flat?&flat_token:&plain_token,flat,__VA_ARGS__)
#define reset(owner,flat,...) jai_fixture_reset(owner,flat?&flat_token:&plain_token,flat,__VA_ARGS__)
#define release(owner,flat,...) jai_fixture_release(owner,flat?&flat_token:&plain_token,flat,__VA_ARGS__)
"#;
