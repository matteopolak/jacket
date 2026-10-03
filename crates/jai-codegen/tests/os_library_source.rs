//! Source-only reference reads; execute only freshly emitted code and installed libc.
#![cfg(all(
    target_pointer_width = "64",
    any(target_os = "macos", target_os = "linux")
))]

#[path = "support/native_tools.rs"]
mod native_tools;
use jai_codegen::optimization::Optimization;
use jai_codegen::target::{NativeTarget, TargetOptions};
use jai_types::BitcodeOptimization;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "jai-os-source-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }
    fn run(&self, source: &str, expected: i32) {
        self.run_with_support(source, expected, None);
    }
    fn run_with_support(&self, source: &str, expected: i32, support: Option<&str>) {
        let path = self.0.join("main.jai");
        fs::write(&path, source).unwrap();
        let support_path = self.0.join("support.c");
        if let Some(support) = support {
            fs::write(&support_path, support).unwrap();
        }
        let graph =
            jai_modules::ModuleGraph::load(&path, jai_modules::GraphOptions::default()).unwrap();
        for bitcode in [BitcodeOptimization::O0, BitcodeOptimization::O2] {
            let target = NativeTarget::select(&TargetOptions {
                optimization: Optimization {
                    bitcode,
                    ..Default::default()
                },
                ..Default::default()
            })
            .unwrap();
            let program = jai_sema::resolve_graph_with_options(
                &graph,
                &jai_sema::ResolveOptions {
                    target: Some(target.build_target().unwrap()),
                    layout: Some(target.layout_policy().unwrap()),
                    ..Default::default()
                },
                &mut jai_vm::NoEffects,
            )
            .unwrap_or_else(|error| panic!("{}", error.render(graph.sources())));
            assert!(
                matches!(
                    jai_vm::execute(&program, jai_vm::Limits::default()).outcome,
                    jai_vm::Outcome::Failed(jai_vm::Error::UnsupportedForeignProcedure(_))
                ),
                "OS foreign calls must not silently execute in the bounded VM"
            );
            let context = jai_codegen::Context::create();
            let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
            let object = self.0.join("program.o");
            let executable = self.0.join("program");
            target.write_object(&module, &object).unwrap();
            let mut command = native_tools::clang_command();
            command
                .arg(&object)
                .arg("-pthread")
                .arg("-o")
                .arg(&executable);
            if support.is_some() {
                command.arg(&support_path);
            }
            if cfg!(target_os = "macos") {
                command.arg("-Wl,-no_fixup_chains");
            }
            let linked = command.output().unwrap();
            assert!(
                linked.status.success(),
                "{}\n{}",
                String::from_utf8_lossy(&linked.stderr),
                module.print_to_string()
            );
            let mut child = Command::new(&executable)
                .current_dir(&self.0)
                .spawn()
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                if let Some(status) = child.try_wait().unwrap() {
                    assert_eq!(
                        status.code(),
                        Some(expected),
                        "{bitcode:?}\n{}",
                        module.print_to_string()
                    );
                    break;
                }
                if Instant::now() >= deadline {
                    child.kill().unwrap();
                    child.wait().unwrap();
                    panic!("OS fixture exceeded ten seconds");
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Copy complete original top-level definitions, without rewriting their bodies.
fn reference_file_definition(source: &str, name: &str) -> String {
    let prefix = format!("{name} :: ");
    // A suffix such as `lock` inside `block` is not the requested definition.
    let start = if source.starts_with(&prefix) {
        0
    } else {
        source.find(&format!("\n{prefix}")).unwrap() + 1
    };
    let remaining = &source[start..];
    let end = remaining.find("\n}").unwrap() + 2;
    remaining[..end].to_owned()
}

/// Original inputs stay outside the checkout and are optional at test runtime.
fn optional_original_source(relative: &str) -> Option<String> {
    let root = std::env::var_os("JAI_RS_OS_SOURCE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let path = root.join(relative);
    match fs::read_to_string(&path) {
        Ok(source) => Some(source),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            eprintln!(
                "SKIP optional original source: {} is absent",
                path.display()
            );
            None
        }
        Err(error) => panic!(
            "cannot read optional original source {}: {error}",
            path.display()
        ),
    }
}

#[test]
fn unchanged_reference_file_procedures_roundtrip_seek_eof_move_and_delete() {
        return;
    };
    let mut fixture = String::from(
        r#"
        FILE :: struct {}
        fopen :: (path:*u8, mode:*u8)->*FILE #foreign "fopen";
        fclose :: (file:*FILE)->s32 #foreign "fclose";
        fread :: (data:*void,size:u64,count:u64,file:*FILE)->u64 #foreign "fread";
        fwrite :: (data:*void,size:u64,count:u64,file:*FILE)->u64 #foreign "fwrite";
        feof :: (file:*FILE)->s32 #foreign "feof";
        fseek :: (file:*FILE,offset:s64,whence:s32)->s32 #foreign "fseek";
        rename :: (old:*u8,new:*u8)->s32 #foreign "rename";
        _remove :: (path:*u8)->s32 #foreign "remove";
        abort :: () #foreign "abort";
        SEEK_SET :: 0;
        temp_c_string :: (value:string)->*u8 { return value.data; }
        assert :: (condition:bool) { if !condition abort(); }
        log_error :: (message:string) { abort(); }
    "#,
    );
    // file_open also needs System error reporting even when log_errors is false;
    // test the remaining original routines with a directly opened opaque FILE.
    for name in [
        "File",
        "is_valid",
        "file_read",
        "file_write",
        "file_set_position",
        "file_move",
        "file_delete",
    ] {
        fixture.push_str(&reference_file_definition(&source, name));
        fixture.push('\n');
    }
    fixture.push_str(
        r#"
        main :: ()->int {
            path := "payload.bin\0";
            mode := "wb+\0";
            handle := fopen(path.data,mode.data);
            file:File; file.handle=handle;
            if !is_valid(file) return 1;
            payload := "abcde";
            if !file_write(*file,cast(*void) payload.data,payload.count) return 2;
            if !file_set_position(file,2) return 3;
            buffer:[16]u8;
            ok,count:=file_read(file,cast(*void) *buffer,16);
            if !ok || count != 3 return 4;
            if buffer[0] != 99 || buffer[1] != 100 || buffer[2] != 101 return 5;
            invalid,zero:=file_read(file,cast(*void) *buffer,0);
            if invalid || zero != 0 return 6;
            if fclose(handle) != 0 return 7;
            if !file_move(path,"moved.bin\0") return 8;
            if !file_delete("moved.bin\0") return 9;
            read_mode := "rb\0";
            missing:=fopen(path.data,read_mode.data);
            if missing != null return 10;
            if file_delete(path) return 11;
            return 42;
        }
    "#,
    );
    Fixture::new().run(&fixture, 42);
}

#[test]
fn source_process_fork_wait_and_exit_status_use_real_posix_calls() {
    Fixture::new().run(
        r#"
        fork :: ()->s32 #foreign "fork";
        waitpid :: (pid:s32,status:*s32,options:s32)->s32 #foreign "waitpid";
        child_exit :: (code:s32) #foreign "_exit";
        pipe :: (handles:*s32)->s32 #foreign "pipe";
        close :: (handle:s32)->s32 #foreign "close";
        write :: (handle:s32,data:*void,count:u64)->s64 #foreign "write";
        read :: (handle:s32,data:*void,count:u64)->s64 #foreign "read";
        main :: ()->int {
            handles:[2]s32;
            if pipe(cast(*s32) *handles) != 0 return 1;
            pid:=fork();
            if pid < 0 return 2;
            if pid == 0 {
                if close(handles[0]) != 0 { child_exit(21); return 3; }
                byte:u8=77;
                if write(handles[1],cast(*void) *byte,1) != 1 { child_exit(22); return 4; }
                child_exit(23); return 5;
            }
            if close(handles[1]) != 0 return 6;
            byte:u8=0;
            if read(handles[0],cast(*void) *byte,1) != 1 || byte != 77 return 7;
            if read(handles[0],cast(*void) *byte,1) != 0 return 8;
            if close(handles[0]) != 0 return 9;
            status:s32=0;
            if waitpid(pid,*status,0) != pid return 10;
            if (status & 127) != 0 return 11;
            if ((status >> 8) & 255) != 23 return 12;
            return 42;
        }
    "#,
        42,
    );
}

#[test]
fn source_pthread_c_callback_mutates_argument_and_join_returns_pointer() {
    // pthread_t is pointer-sized on tested 64-bit POSIX hosts; storage is opaque
    // and passed back unchanged. No guessed pthread mutex/attribute layout.
    Fixture::new().run(r#"
        Callback :: #type (argument:*void)->*void #c_call;
        pthread_create :: (thread:*u64,attributes:*void,callback:Callback,argument:*void)->s32 #foreign "pthread_create";
        pthread_join :: (thread:u64,result:**void)->s32 #foreign "pthread_join";
        worker :: (argument:*void)->*void #c_call {
            value:=cast(*s64) argument;
            value.* += 19;
            return argument;
        }
        main :: ()->int {
            thread:u64=0;
            value:s64=23;
            pointer:=cast(*void) *value;
            if pthread_create(*thread,null,worker,pointer) != 0 return 1;
            returned:*void=null;
            if pthread_join(thread,*returned) != 0 return 2;
            if returned != pointer return 3;
            return cast(int) value;
        }
    "#,42);
}

#[test]
fn source_thread_initializer_local_c_callback_is_callable_from_pthread() {
    Fixture::new().run(
        r#"
        Callback :: #type (argument:*void)->*void #c_call;
        pthread_create :: (thread:*u64,attributes:*void,callback:Callback,argument:*void)->s32 #foreign "pthread_create";
        pthread_join :: (thread:u64,result:**void)->s32 #foreign "pthread_join";
        start :: (thread:*u64, argument:*void)->bool {
            entry_proc :: (parameter:*void)->*void #c_call {
                value:=cast(*s64) parameter;
                value.*+=19;
                return parameter;
            }
            return pthread_create(thread,null,entry_proc,argument) == 0;
        }
        main :: ()->int {
            thread:u64=0;
            value:s64=23;
            argument:=cast(*void) *value;
            if !start(*thread,argument) return 1;
            returned:*void=null;
            if pthread_join(thread,*returned) != 0 || returned != argument return 2;
            return cast(int) value;
        }
        "#,
        42,
    );
}

/// The adapter owns real pthread objects so Jai never guesses their host layout.
const OPAQUE_PTHREAD_SUPPORT: &str = r#"
#include <errno.h>
#include <pthread.h>
#include <stdlib.h>

static int live_objects;
int jai_test_mutex_create(void **out) {
    pthread_mutex_t *mutex = malloc(sizeof(*mutex));
    if (!mutex) return ENOMEM;
    int result = pthread_mutex_init(mutex, NULL);
    if (result) { free(mutex); return result; }
    *out = mutex;
    ++live_objects;
    return 0;
}
int jai_test_condition_create(void **out) {
    pthread_cond_t *condition = malloc(sizeof(*condition));
    if (!condition) return ENOMEM;
    int result = pthread_cond_init(condition, NULL);
    if (result) { free(condition); return result; }
    *out = condition;
    ++live_objects;
    return 0;
}
int jai_test_mutex_destroy(void **handle) {
    int result = pthread_mutex_destroy(*handle);
    if (!result) { free(*handle); *handle = NULL; --live_objects; }
    return result;
}
int jai_test_condition_destroy(void **handle) {
    int result = pthread_cond_destroy(*handle);
    if (!result) { free(*handle); *handle = NULL; --live_objects; }
    return result;
}
int jai_test_mutex_lock(void **handle) { return pthread_mutex_lock(*handle); }
int jai_test_mutex_unlock(void **handle) { return pthread_mutex_unlock(*handle); }
int jai_test_condition_broadcast(void **handle) { return pthread_cond_broadcast(*handle); }
int jai_test_condition_wait(void **condition, void **mutex) {
    return pthread_cond_wait(*condition, *mutex);
}
int jai_test_live_objects(void) { return live_objects; }
int jai_test_errno(void) { return errno; }
int jai_test_bad_descriptor(void) { return EBADF; }
"#;

#[test]
fn reference_blocker_helpers_synchronize_async_io_context_errors_and_teardown() {
    else {
        return;
    };
    let focus = optional_original_source(
        "corpus/upstream/focus-editor--focus/modules/File_Async/thread_pool.jai",
    );
    let mut helpers = String::new();
    for name in ["wake_up", "block", "lock", "unlock"] {
        let definition = reference_file_definition(&source, name);
        if let Some(focus) = &focus {
            assert_eq!(
                definition.replace("\r\n", "\n"),
                reference_file_definition(focus, name).replace("\r\n", "\n"),
                "pinned reference and Focus {name} definitions differ"
            );
        }
        helpers.push_str(&definition);
        helpers.push('\n');
    }
    run_async_worker(&helpers);
}

#[test]
fn authored_async_worker_synchronizes_context_completions_errors_and_teardown() {
    // Independently authored wrappers keep the protocol mandatory without the corpus.
    run_async_worker(
        r#"
        wake_up :: (b:*Blocker) {
            pthread_mutex_lock(*b.lock);
            pthread_cond_broadcast(*b.condition);
            pthread_mutex_unlock(*b.lock);
        }
        block :: (b:*Blocker) { pthread_cond_wait(*b.condition,*b.lock); }
        lock :: (b:*Blocker) { pthread_mutex_lock(*b.lock); }
        unlock :: (b:*Blocker) { pthread_mutex_unlock(*b.lock); }
    "#,
    );
}

fn run_async_worker(helpers: &str) {
    let mut fixture = String::from(
        r#"
        #add_context worker_number: s64 = 7;
        Blocker :: struct { lock:*void; condition:*void; }
        pthread_mutex_lock :: (lock:**void)->s32 #foreign "jai_test_mutex_lock";
        pthread_mutex_unlock :: (lock:**void)->s32 #foreign "jai_test_mutex_unlock";
        pthread_cond_broadcast :: (condition:**void)->s32 #foreign "jai_test_condition_broadcast";
        pthread_cond_wait :: (condition:**void, lock:**void)->s32 #foreign "jai_test_condition_wait";
        mutex_create :: (lock:**void)->s32 #foreign "jai_test_mutex_create";
        mutex_destroy :: (lock:**void)->s32 #foreign "jai_test_mutex_destroy";
        condition_create :: (condition:**void)->s32 #foreign "jai_test_condition_create";
        condition_destroy :: (condition:**void)->s32 #foreign "jai_test_condition_destroy";
        live_objects :: ()->s32 #foreign "jai_test_live_objects";
        errno :: ()->s32 #foreign "jai_test_errno";
        bad_descriptor :: ()->s32 #foreign "jai_test_bad_descriptor";
        Callback :: #type (argument:*void)->*void #c_call;
        pthread_create :: (thread:*u64,attributes:*void,callback:Callback,argument:*void)->s32 #foreign "pthread_create";
        pthread_join :: (thread:u64,result:**void)->s32 #foreign "pthread_join";
        FILE :: struct {}
        fopen :: (path:*u8, mode:*u8)->*FILE #foreign "fopen";
        fclose :: (file:*FILE)->s32 #foreign "fclose";
        fileno :: (file:*FILE)->s32 #foreign "fileno";
        pread :: (descriptor:s32,data:*void,count:u64,offset:s64)->s64 #foreign "pread";
        pwrite :: (descriptor:s32,data:*void,count:u64,offset:s64)->s64 #foreign "pwrite";
        Action :: enum { IDLE; WRITE; READ; FAIL; STOP; }
        Job :: struct {
            blocker:Blocker;
            started:bool;
            state:Action;
            descriptor:s32;
            count:s64;
            error:s32;
            cookie:s64;
            observed_context:s64;
            default_context:s64;
            saved_context:#Context;
            bytes:[16]u8;
        }
    "#,
    );
    fixture.push_str(helpers);
    fixture.push_str(r#"
        observe_context :: ()->s64 { context.worker_number += 1; return context.worker_number; }
        worker :: (argument:*void)->*void #c_call {
            job:=cast(*Job) argument;
            // A C callback can explicitly establish a fresh default context.
            push_context { job.default_context=context.worker_number; }
            // Match Thread's suspended start: publication follows pthread_create.
            push_context {
                lock(*job.blocker);
                while !job.started block(*job.blocker);
                unlock(*job.blocker);
            }
            push_context job.saved_context {
                while true {
                    lock(*job.blocker);
                    while job.state == .IDLE block(*job.blocker);
                    action:=job.state;
                    if action == .STOP { unlock(*job.blocker); return argument; }
                    if action == .WRITE {
                        payload:="abcde";
                        job.count=pwrite(job.descriptor,cast(*void) payload.data,5,0);
                    } else if action == .READ {
                        job.count=pread(job.descriptor,cast(*void) *job.bytes,16,2);
                    } else {
                        job.count=pread(-1,cast(*void) *job.bytes,1,0);
                    }
                    job.error=0;
                    if job.count < 0 job.error=errno();
                    job.observed_context=observe_context();
                    job.cookie+=100;
                    job.state=.IDLE;
                    unlock(*job.blocker);
                    wake_up(*job.blocker);
                }
            }
            return null;
        }
        request :: (job:*Job, action:Action, cookie:s64) {
            lock(*job.blocker);
            job.cookie=cookie;
            job.state=action;
            unlock(*job.blocker);
            wake_up(*job.blocker);
            lock(*job.blocker);
            while job.state != .IDLE block(*job.blocker);
            unlock(*job.blocker);
        }
        main :: ()->int {
            path:="async.bin\0";
            mode:="wb+\0";
            file:=fopen(path.data,mode.data);
            if file == null return 1;
            job:Job;
            job.descriptor=fileno(file);
            if mutex_create(*job.blocker.lock) != 0 return 2;
            if condition_create(*job.blocker.condition) != 0 return 3;
            if live_objects() != 2 return 4;
            thread:u64=0;
            argument:=cast(*void) *job;
            if pthread_create(*thread,null,worker,argument) != 0 return 5;
            lock(*job.blocker);
            job.saved_context=context;
            job.saved_context.worker_number=40;
            job.started=true;
            unlock(*job.blocker);
            wake_up(*job.blocker);
            request(*job,.WRITE,1);
            if job.count != 5 || job.error != 0 || job.cookie != 101 return 6;
            if job.observed_context != 41 return 7;
            request(*job,.READ,2);
            if job.count != 3 || job.error != 0 || job.cookie != 102 return 8;
            if job.bytes[0] != 99 || job.bytes[1] != 100 || job.bytes[2] != 101 return 9;
            if job.observed_context != 42 return 10;
            request(*job,.FAIL,3);
            if job.count != -1 || job.error != bad_descriptor() || job.cookie != 103 return 11;
            if job.observed_context != 43 return 12;
            lock(*job.blocker);
            job.state=.STOP;
            unlock(*job.blocker);
            wake_up(*job.blocker);
            result:*void=null;
            if pthread_join(thread,*result) != 0 || result != argument return 13;
            if context.worker_number != 7 || job.saved_context.worker_number != 40 || job.default_context != 7 return 14;
            if condition_destroy(*job.blocker.condition) != 0 return 15;
            if mutex_destroy(*job.blocker.lock) != 0 return 16;
            if live_objects() != 0 || job.blocker.lock != null || job.blocker.condition != null return 17;
            if fclose(file) != 0 return 18;
            return 42;
        }
    "#);
    Fixture::new().run_with_support(&fixture, 42, Some(OPAQUE_PTHREAD_SUPPORT));
}
