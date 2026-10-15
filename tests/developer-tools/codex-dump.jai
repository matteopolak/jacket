#import "Basic";
#import "File";
C :: #import "Codex";

errors: s64;

capture :: (text: string, data: *void, info: Log_Info) {
    if info.common_flags & .ERROR errors += 1;
}

main :: () {
    assert(C.dump_file(tprint("%../../target/profiler-codex-export.codex", #filepath)));
    invalid := tprint("%../../target/profiler-codex-invalid-header.codex", #filepath);

    assert(write_entire_file(invalid, "bad"));

    push_context {
        context.logger = capture;
        assert(!C.dump_file(invalid));
    }

    assert(errors == 1);
    print("ok\n");
}
