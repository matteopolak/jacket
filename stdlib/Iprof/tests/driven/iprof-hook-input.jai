#import "Basic";

calls: s64;

client_frame :: () {
    calls += 1;
}

client_editor :: (filename: string, line: s64) {
    assert(filename == "héllo.jai" && line == 42);
    calls += 10;
}

main :: () {
    runtime_frame();
    runtime_editor("héllo.jai", 41);
    optional_hook();
    assert(calls == 11);
    print("ok\n");
}
