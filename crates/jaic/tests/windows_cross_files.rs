//! Target-side metaprograms must use real host files through the target Windows API.
use jaic::build::{BuildEnv, Workspaces};
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use jaic::target::TargetOs;
use std::{path::PathBuf, rc::Rc};

#[test]
fn windows_target_run_reads_and_walks_real_host_files() {
    if cfg!(windows) {
        return;
    }
    let temp = test_support::TempDir::new("windows-cross-files");
    let input = temp.join("input-é-😀.txt");
    std::fs::write(&input, "host bytes: café 😀").unwrap();
    std::fs::create_dir(temp.join("nested")).unwrap();
    std::fs::write(temp.join("nested/child.txt"), "child").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&*temp, temp.join("nested/cycle")).unwrap();
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.os = TargetOs::Windows;
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let fs = Rc::new(NativeFs);
    let workspaces = Workspaces::new(BuildEnv {
        fs: fs.clone(),
        options: options.clone(),
        backend: None,
        unwritten_output_hint: None,
        command_line: Vec::new(),
        make_host: Box::new(|_| Box::new(jaic::interp::NativeHost)),
        report: Box::new(|_, message| panic!("unexpected diagnostic: {message}")),
        observer: None,
    });
    let mut compiler = Compiler::new(options, fs);
    compiler.attach_workspaces(workspaces);
    let source = format!(
        r#"
        #import "Basic";
        #import "File";
        #import "File_Utilities";
        #import "Compiler";
        #run {{
            // Access target compiler state so this run cannot succeed through the host twin.
            root, nodes := compiler_get_nodes(#code 1);
            assert(root != null);
            assert(OS == .WINDOWS && BUILD_OS != .WINDOWS);
            text, ok := read_entire_file("{input}");
            assert(ok && text == "host bytes: café 😀");
            free(text);
            assert(write_entire_file("{output}", "written on host"));
            assert(copy_file("{output}", "{copy}"));
            assert(file_link("{link}", "{copy}"));
            file, opened := file_open("{input}");
            assert(opened);
            assert(file_seek(file, 5, .START));
            tail, read_ok := read_entire_file(file);
            assert(read_ok && tail == "bytes: café 😀");
            free(tail);
            assert(file_get_canonical_path(file) != "");
            file_close(*file);
            time, size, found := file_modtime_and_size("{input}");
            assert(found && size == 22);
            count: int;
            assert(visit_files("{dir}", true, *count, (info: *File_Visit_Info, count: *int) {{ count.* += 1; }}));
            assert(count == {entry_count});
            mapped, mapped_ok := map_file("{input}");
            assert(mapped_ok && mapped.data == "host bytes: café 😀");
            assert(munmap_file(*mapped));
            assert(munmap_file(*mapped));
            assert(compare_apollo_times(current_time_monotonic(), APOLLO_TIME_INVALID) != 0);
            assert(current_time_consensus().low != 0);
            _, missing := file_open("{dir}/missing.txt");
            assert(!missing);
        }}
        main :: () {{}}
    "#,
        input = input.display(),
        output = temp.join("output.txt").display(),
        copy = temp.join("copy.txt").display(),
        link = temp.join("link.txt").display(),
        dir = temp.display(),
        entry_count = if cfg!(unix) {
            6
        } else {
            5
        }
    );
    compiler
        .compile_sources(&[ProgramSource::String(source)])
        .unwrap_or_else(|error| panic!("{}", error.render(&compiler.src.sources)));
    assert_eq!(
        std::fs::read_to_string(temp.join("output.txt")).unwrap(),
        "written on host"
    );
    assert_eq!(
        std::fs::read_to_string(temp.join("link.txt")).unwrap(),
        "written on host"
    );
}
