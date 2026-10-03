//! Authored source, real installed POSIX errors, and freshly generated native code.
#![cfg(all(
    target_pointer_width = "64",
    any(target_os = "macos", target_os = "linux")
))]

#[path = "support/native_tools.rs"]
mod native_tools;
use jai_codegen::{
    optimization::Optimization,
    target::{NativeTarget, TargetOptions},
};
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
            "jai-os-resources-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }
    fn run(&self, source: &str, support: &str) {
        let path = self.0.join("main.jai");
        let support_path = self.0.join("support.c");
        fs::write(&path, source).unwrap();
        fs::write(&support_path, support).unwrap();
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
                "native OS effects must not silently execute in the bounded VM"
            );
            let context = jai_codegen::Context::create();
            let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
            let object = self.0.join("program.o");
            let executable = self.0.join("program");
            target.write_object(&module, &object).unwrap();
            let mut command = native_tools::clang_command();
            command
                .arg(&object)
                .arg(&support_path)
                .arg("-pthread")
                .arg("-o")
                .arg(&executable);
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
                        Some(42),
                        "{bitcode:?}\n{}",
                        module.print_to_string()
                    );
                    break;
                }
                if Instant::now() >= deadline {
                    child.kill().unwrap();
                    child.wait().unwrap();
                    panic!("OS resource fixture exceeded ten seconds");
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

#[test]
fn stream_write_error_and_eof_are_distinct_and_clearerr_resets_them() {
    Fixture::new().run(
        r#"
        FILE :: struct {}
        fopen :: (path:*u8,mode:*u8)->*FILE #foreign "fopen";
        fclose :: (file:*FILE)->s32 #foreign "fclose";
        fread :: (data:*void,size:u64,count:u64,file:*FILE)->u64 #foreign "fread";
        fwrite :: (data:*void,size:u64,count:u64,file:*FILE)->u64 #foreign "fwrite";
        ferror :: (file:*FILE)->s32 #foreign "ferror";
        feof :: (file:*FILE)->s32 #foreign "feof";
        clearerr :: (file:*FILE) #foreign "clearerr";
        fseek :: (file:*FILE,offset:s64,whence:s32)->s32 #foreign "fseek";
        errno_code :: ()->s32 #foreign "fixture_errno";
        bad_descriptor_code :: ()->s32 #foreign "fixture_ebadf";
        read :: (descriptor:s32,data:*void,count:u64)->s64 #foreign "read";
        main :: ()->int {
            path:="payload.bin\0";
            write_mode:="wb\0";
            write_file:=fopen(path.data,write_mode.data);
            if write_file == null return 1;
            bytes:="z";
            if fwrite(cast(*void) bytes.data,1,1,write_file) != 1 return 2;
            if fclose(write_file) != 0 return 3;
            read_mode:="rb\0";
            read_file:=fopen(path.data,read_mode.data);
            if read_file == null return 4;
            if fwrite(cast(*void) bytes.data,1,1,read_file) != 0 return 5;
            if ferror(read_file) == 0 || feof(read_file) != 0 return 6;
            clearerr(read_file);
            if ferror(read_file) != 0 || feof(read_file) != 0 return 7;
            buffer:[2]u8;
            if fread(cast(*void) *buffer,1,2,read_file) != 1 return 8;
            if buffer[0] != 122 || feof(read_file) == 0 || ferror(read_file) != 0 return 9;
            clearerr(read_file);
            if feof(read_file) != 0 || ferror(read_file) != 0 return 10;
            if fseek(read_file,0,0) != 0 return 11;
            if fread(cast(*void) *buffer,1,1,read_file) != 1 || buffer[0] != 122 return 12;
            if fclose(read_file) != 0 return 13;
            expected:=bad_descriptor_code();
            if read(-1,cast(*void) *buffer,1) != -1 return 14;
            if errno_code() != expected return 15;
            return 42;
        }
    "#,
        r#"
        #include <errno.h>
        int fixture_errno(void) { return errno; }
        int fixture_ebadf(void) { return EBADF; }
    "#,
    );
}

#[test]
fn joined_thread_transfers_owned_allocation_before_caller_releases_it() {
    Fixture::new().run(r#"
        Payload :: struct { value:s64; }
        Callback :: #type (argument:*void)->*void #c_call;
        pthread_create :: (thread:*u64,attributes:*void,callback:Callback,argument:*void)->s32 #foreign "pthread_create";
        pthread_join :: (thread:u64,result:**void)->s32 #foreign "pthread_join";
        allocate :: (bytes:u64)->*void #foreign "fixture_allocate";
        release :: (memory:*void) #foreign "fixture_release";
        outstanding :: ()->s32 #foreign "fixture_outstanding";
        worker :: (argument:*void)->*void #c_call {
            input:=cast(*s64) argument;
            payload:=cast(*Payload) allocate(8);
            if payload == null return null;
            payload.value=input.* + 19;
            return cast(*void) payload;
        }
        main :: ()->int {
            if outstanding() != 0 return 1;
            thread:u64=0;
            input:s64=23;
            if pthread_create(*thread,null,worker,cast(*void) *input) != 0 return 2;
            returned:*void=null;
            if pthread_join(thread,*returned) != 0 || returned == null return 3;
            if outstanding() != 1 return 4;
            payload:=cast(*Payload) returned;
            result:=payload.value;
            release(returned);
            if outstanding() != 0 return 5;
            return cast(int) result;
        }
    "#, r#"
        #include <stdlib.h>
        #include <stdatomic.h>
        static _Atomic int outstanding;
        void *fixture_allocate(unsigned long long bytes) {
            void *memory = malloc((size_t) bytes);
            if (memory) atomic_fetch_add(&outstanding, 1);
            return memory;
        }
        void fixture_release(void *memory) {
            if (memory) { free(memory); atomic_fetch_sub(&outstanding, 1); }
        }
        int fixture_outstanding(void) { return atomic_load(&outstanding); }
    "#);
}
