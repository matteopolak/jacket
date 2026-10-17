// Trusted independent unit tests; public facade intentionally rejects this Mac host.
#load "../../internal/build/linux-link.jai";

main :: () {
    configured := toolchain_root();

    print("root=%\n", configured);
    free(configured);
    root := normalize_root("C:\\independent SDK\\\\");

    assert(root == "C:/independent SDK");
    free(root);
    empty := normalize_root("////");

    assert(empty.count == 0);
    free(empty);

    phase: C.Message_Phase;
    compiled := string.["first object.o", "second.o"];
    support := string.["runtime.o"];
    users := string.["user", "folder/existing.a", "shared.so"];
    systems := string.["libm.dll", "C:\\SDK\\kernel32.dll", "ignored.so", "-lc", "-lpthread", "ignored"];

    phase.compiler_generated_object_files = compiled;
    phase.support_object_files = support;
    phase.user_libraries = users;
    phase.system_libraries = systems;
    args := link_arguments("C:/independent SDK", *phase, "output space/program");

    defer release_arguments(*args);
    expected := string.[
        "C:/independent SDK/bin/ld.lld.exe", "--sysroot", "C:/independent SDK/x86_64-unknown-linux-gnu",
        "--build-id", "--gdb-index", "-g", "--nostdlib",
        "-L", "C:/independent SDK/x86_64-unknown-linux-gnu/usr/lib",
        "-L", "C:/independent SDK/x86_64-unknown-linux-gnu/usr/lib64",
        "C:/independent SDK/x86_64-unknown-linux-gnu/usr/lib/crt1.o",
        "C:/independent SDK/x86_64-unknown-linux-gnu/usr/lib/crti.o",
        "C:/independent SDK/x86_64-unknown-linux-gnu/usr/lib/crtn.o",
        "--dynamic-linker", "/lib64/ld-linux-x86-64.so.2", "--eh-frame-hdr", "-export-dynamic", "-rpath=$ORIGIN",
        "first object.o", "second.o", "runtime.o", "--start-group", "user.a", "folder/existing.a", "shared.so",
        "-lm", "kernel32.a", "-lc", "-lpthread", "--end-group", "-o", "output space/program",
    ];

    assert(args.count == expected.count);
    for arg, index: args assert(arg == expected[index]);

    p := get_plugin();

    assert(p != null);
    assert(!p.init && p.before_intercept && p.add_source && p.message && p.finish && p.shutdown);
    assert(!p.log_help && !p.communication_proc);
    add_source(p);
    finish(p);
    ignored: C.Message;
    ignored.kind = .FILE;
    message(p, *ignored);
    phase.kind = .PHASE;
    phase.phase = .PRE_WRITE_EXECUTABLE;
    phase.executable_name = "target";
    message(p, *phase);
    shutdown(p);
    write_string("ok\n");
}
