// `USAGE_MODE` is accepted as a module parameter in each of its forms, and the module's
// declarations are the same in all of them.
#import "Basic";
Reader :: #import "Codex"(USAGE_MODE = .READ);
Writer :: #import "Codex"(USAGE_MODE = .WRITE);
Both :: #import "Codex"(USAGE_MODE = .READ | .WRITE);
Default :: #import "Codex";

main :: () {
    assert(type_of(Reader.dump_file) == type_of(Writer.dump_file));
    assert(type_of(Both.get_plugin) == type_of(Default.get_plugin));
    assert(size_of(Reader.Header_Record) == size_of(Writer.Header_Record));
    print("ok\n");
}
