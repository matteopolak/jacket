//! Time batches of own generated native code, including process startup.
use super::mixed_module::{Fixture, target};
use divan::{Bencher, counter::ItemsCount};
use jai_codegen::optimization::BitcodeOptimization;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
};

#[path = "../../../jai-codegen/tests/support/native_tools.rs"]
#[allow(dead_code)]
mod native_tools;

const BATCH: usize = 256;

struct Scratch(PathBuf, bool);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let retained = std::env::var_os("JAI_BENCH_NATIVE_ARTIFACTS").map(PathBuf::from);
        let root = retained.clone().unwrap_or_else(std::env::temp_dir);
        fs::create_dir_all(&root).unwrap();
        let path = root.join(format!(
            "jai-own-native-bench-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir(&path).unwrap();
        Self(path, retained.is_some())
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        if !self.1 {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

#[divan::bench(args = [4, 64, 1024])]
fn mixed_native_batch_o0(bencher: Bencher, count: usize) {
    runtime(bencher, count, BitcodeOptimization::O0);
}
#[divan::bench(args = [4, 64, 1024])]
fn mixed_native_batch_o2(bencher: Bencher, count: usize) {
    runtime(bencher, count, BitcodeOptimization::O2);
}

fn runtime(bencher: Bencher, count: usize, level: BitcodeOptimization) {
    let fixture = Fixture::new(count);
    let target = target(level);
    assert!(target.is_host());
    let program = fixture.program(&target);
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower_library_for_target(
        &context,
        program.library(),
        &jai_codegen::native_reachability::Publication::Selected(vec![]),
        &target,
    )
    .unwrap();
    let scratch = Scratch::new();
    let object = scratch.0.join("own-generated.o");
    let source = scratch.0.join("own-harness.c");
    let executable = scratch.0.join("own-generated-program");
    target.write_object(&module, &object).unwrap();
    fs::write(&source, format!(
        "#include <stdint.h>\n#include <stdio.h>\n#include <sys/resource.h>\nextern int64_t bench_compute(int64_t);\nint main(int argc, char **argv) {{ (void)argv; for(int64_t i=0;i<{BATCH};++i) {{ int64_t seed=i%17; int64_t expected={count}*seed+({count}LL*({count}-1))/2+(({count}+1)/2)*16+({count}/2)*7; if(bench_compute(seed)!=expected) return 1; }} if(argc>1) {{ struct rusage usage; if(getrusage(RUSAGE_SELF,&usage)) return 2;\n#ifdef __APPLE__\nprintf(\"%llu\\n\",(unsigned long long)usage.ru_maxrss);\n#else\nprintf(\"%llu\\n\",(unsigned long long)usage.ru_maxrss*1024);\n#endif\n}} return 0; }}\n"
    )).unwrap();
    // Resolve against the actual input root even in an isolated benchmark snapshot.
    let repository = std::env::var_os("JAI_BENCH_CORPUS_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let explicit = std::env::var_os("JAI_RS_CLANG");
    let prefix = std::env::var_os("LLVM_SYS_221_PREFIX").map(PathBuf::from);
    let search = std::env::var_os("PATH");
    let clang = native_tools::resolve_clang(
        explicit.as_deref(),
        prefix.as_deref(),
        search.as_deref(),
        &repository,
    )
    .unwrap();
    let mut linker = Command::new(&clang);
    scrub(&mut linker);
    let build = linker
        .arg(&object)
        .arg(&source)
        .arg("-O2")
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let execute = || {
        let mut command = Command::new(&executable);
        scrub(&mut command);
        let status = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap();
        assert!(
            status.success(),
            "own native checksum batch failed: {status}"
        );
    };
    execute();
    let mut resource_probe = Command::new(&executable);
    scrub(&mut resource_probe);
    let resources = resource_probe.arg("resources").output().unwrap();
    assert!(resources.status.success());
    let peak_rss: u64 = std::str::from_utf8(&resources.stdout)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    eprintln!(
        "JAI_BENCH_WORKLOAD {{\"name\":\"mixed_native_batch_{level:?}\",\"size\":{count},\"batch_calls\":{BATCH},\"logical_handler_calls\":{},\"expected_outcome\":\"complete\",\"includes_process_startup\":true,\"native_preflight_peak_rss_bytes\":{peak_rss},\"clang\":{:?},\"native_artifacts\":[{:?},{:?},{:?}]}}",
        BATCH * count,
        clang.to_string_lossy(),
        source.to_string_lossy(),
        object.to_string_lossy(),
        executable.to_string_lossy()
    );
    bencher
        .counter(ItemsCount::new(BATCH * count))
        .bench_local(execute);
}

fn scrub(command: &mut Command) {
    for variable in [
        "LIBRARY_PATH",
        "LD_LIBRARY_PATH",
        "LD_PRELOAD",
        "LD_AUDIT",
        "DYLD_LIBRARY_PATH",
        "DYLD_FALLBACK_LIBRARY_PATH",
        "DYLD_FRAMEWORK_PATH",
        "DYLD_FALLBACK_FRAMEWORK_PATH",
        "DYLD_INSERT_LIBRARIES",
        "DYLD_ROOT_PATH",
        "DYLD_VERSIONED_LIBRARY_PATH",
        "DYLD_VERSIONED_FRAMEWORK_PATH",
        "CCC_OVERRIDE_OPTIONS",
        "SDKROOT",
        "CPATH",
        "C_INCLUDE_PATH",
        "CPLUS_INCLUDE_PATH",
        "OBJC_INCLUDE_PATH",
    ] {
        command.env_remove(variable);
    }
}
