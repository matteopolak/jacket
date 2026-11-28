//! Compile-time code built for another C library than the host's is answered in the target's
//! ABI, or refused, never handed to the host's C library as if the two were the same.
use std::{path::PathBuf, rc::Rc};

use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use jaic::target::{TargetCpu, TargetOs};

/// A POSIX target whose C library is not the host's.
fn other_c_library() -> TargetOs {
    if cfg!(target_os = "linux") {
        TargetOs::MacOS
    } else {
        TargetOs::Linux
    }
}

fn compile(os: TargetOs, body: &str) -> Result<(), String> {
    compile_for(os, None, "", body)
}

/// Compile `body` as compile-time code for `os` (and `cpu`, or the host's), after the
/// top-level declarations `head`.
fn compile_for(os: TargetOs, cpu: Option<TargetCpu>, head: &str, body: &str) -> Result<(), String> {
    let stdlib = test_support::repo::stdlib();
    let mut options = Options::host();
    let cpu = cpu.unwrap_or(options.cpu);

    jaic::target::select_os_cpu(&mut options, Some(os), Some(cpu));
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    let source = format!(
        r#"
        #import "Basic";
        #import "File_Utilities";
        {head}
        // A `#no_reset` global keeps every run on the target compiler.
        #no_reset ran := false;
        #run {{
            {body}
            ran = true;
        }}
        main :: () {{}}
    "#
    );

    compiler
        .compile_sources(&[ProgramSource::String(source)])
        .map_err(|error| error.render(&compiler.src.sources))
}

/// A fresh, empty directory for one test.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("jaic-foreign-c-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);

    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// File calls of the target's C library, checked against what they did to the host's files,
/// with the target's flags, records and error numbers.
const FILE_CALLS: &str = r#"
    #import "POSIX";
    #import "File";
    // Linux declares `open` only as `open64`.
    #if OS == .LINUX { open_file :: open64; } else { open_file :: open; }

    check_target_c_library :: (dir: string) {
        assert(is_directory(dir), "is_directory");
        path := tprint("%/a.txt", dir);
        assert(write_entire_file(path, "hello"), "write_entire_file");
        assert(file_exists(path), "file_exists");
        text, read := read_entire_file(path);
        assert(read && text == "hello", "read_entire_file");
        _, size, found := file_modtime_and_size(path);
        assert(found && size == 5, "file_modtime_and_size: % %", found, size);
        names := file_list(dir);
        assert(names.count == 1, "file_list: %", names);

        info: stat_t;
        assert(stat(temp_c_string(path), *info) == 0 && info.st_size == 5, "stat size");
        assert(S_ISREG(info.st_mode), "stat mode");
        assert(stat(temp_c_string(dir), *info) == 0 && S_ISDIR(info.st_mode), "stat dir");
        assert(stat(temp_c_string(tprint("%/missing", dir)), *info) == -1, "stat missing");
        assert(errno() == ENOENT, "errno after stat: %", errno());

        // Linux and macOS number this error differently.
        assert(rmdir(temp_c_string(dir)) == -1 && errno() == ENOTEMPTY, "rmdir: %", errno());

        fd := open_file(temp_c_string(path), O_CREAT | O_EXCL | O_WRONLY, cast(u32) 420);
        assert(fd == -1 && errno() == EEXIST, "exclusive open: % %", fd, errno());
        made := tprint("%/b.txt", dir);
        fd = open_file(temp_c_string(made), O_CREAT | O_TRUNC | O_WRONLY, cast(u32) 420);
        assert(fd >= 0, "create: %", errno());
        assert(write(fd, "four".data, 4) == 4, "write");
        assert(close(fd) == 0, "close");
        fd = open_file(temp_c_string(made), O_WRONLY | O_APPEND);
        assert(fd >= 0 && write(fd, "!".data, 1) == 1 && close(fd) == 0, "append");

        fd = open_file(temp_c_string(made), O_RDONLY);
        assert(fd >= 0, "open for fstat");
        assert(fstat(fd, *info) == 0 && info.st_size == 5, "fstat size: %", info.st_size);
        close(fd);

        // A relative path resolves against a directory descriptor, not the working directory.
        directory := open_file(temp_c_string(dir), O_RDONLY);
        assert(directory >= 0, "open the directory: %", errno());
        #if OS == .LINUX {
            found_at := syscall(SYS_newfstatat, directory, temp_c_string("a.txt"), *info, 0);
        } else {
            found_at := fstatat(directory, temp_c_string("a.txt"), *info, 0);
        }
        assert(found_at == 0 && info.st_size == 5, "fstatat with a directory: % %", found_at, info.st_size);
        close(directory);

        // `mkdtemp` fills the template in place and means the same in both libraries.
        template := tprint("%/tmp-XXXXXX\0", dir);
        assert(mkdtemp(template.data) != null, "mkdtemp: %", errno());
        assert(template[template.count - 7] != #char "X", "mkdtemp replaced the template");
        assert(is_directory(string.{ count = template.count - 1, data = template.data }), "mkdtemp made a directory");
    }
"#;

#[test]
fn file_calls_of_another_c_library_are_answered_in_its_abi() {
    let cpus: &[Option<TargetCpu>] = if other_c_library() == TargetOs::Linux {
        // The stat family is a system call on both, with each CPU's own numbers.
        &[Some(TargetCpu::X64), Some(TargetCpu::Arm64)]
    } else {
        &[None]
    };

    for &cpu in cpus {
        let dir = scratch(&format!("{cpu:?}"));
        let result = compile_for(
            other_c_library(),
            cpu,
            FILE_CALLS,
            &format!(r#"check_target_c_library("{}");"#, dir.display()),
        );
        let _ = std::fs::remove_dir_all(&dir);

        result.unwrap_or_else(|error| panic!("{cpu:?}: {error}"));
    }
}

#[test]
fn a_target_c_library_call_with_no_answer_on_another_host_is_refused() {
    // `struct utsname` has other field sizes on Linux and macOS.
    let error = compile_for(
        other_c_library(),
        None,
        r#"#import "POSIX";"#,
        "name: utsname; uname(*name);",
    )
    .unwrap_err();

    assert!(
        error.contains("which the C library of this") && error.contains("does not provide"),
        "{error}"
    );
}

#[test]
fn a_windows_c_runtime_call_on_another_host_is_refused_unless_identical() {
    if cfg!(windows) {
        return;
    }

    let head = r#"
        crt :: #system_library "libc";
        c_fopen :: (path: *u8, mode: *u8) -> *void #foreign crt "fopen";
        c_strlen :: (text: *u8) -> u64 #foreign crt "strlen";
    "#;
    // The C runtime's `FILE*` streams translate line ends in text mode; the host's do not.
    let file = env!("CARGO_MANIFEST_DIR");
    let error = compile_for(
        TargetOs::Windows,
        None,
        head,
        &format!(r#"c_fopen("{file}/Cargo.toml", "r");"#),
    )
    .unwrap_err();

    assert!(
        error.contains("built for Windows calls `fopen`") && error.contains("does not provide"),
        "{error}"
    );
    compile_for(
        TargetOs::Windows,
        None,
        head,
        r#"assert(c_strlen("four") == 4);"#,
    )
    .unwrap_or_else(|error| panic!("{error}"));
}

#[test]
fn calls_every_c_library_makes_the_same_way_still_run() {
    // Allocation, strings and output reach the C library as on the host.
    compile(
        other_c_library(),
        r#"text := tprint("% %", 1, "two"); assert(text == "1 two"); write_string("");"#,
    )
    .unwrap_or_else(|error| panic!("{error}"));
}

#[test]
fn compile_time_code_for_another_c_library_finds_the_compiler_as_its_executable() {
    // The executable compile-time code runs in is the compiler, whichever OS the target is.
    for os in [TargetOs::Linux, TargetOs::MacOS, TargetOs::Windows] {
        compile(
            os,
            r#"#import "System"; path := get_path_of_running_executable(); assert(path.count > 0, "no path");"#,
        )
        .unwrap_or_else(|error| panic!("{os:?}: {error}"));
    }
}
