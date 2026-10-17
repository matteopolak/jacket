// This private named native pasteboard never accesses the user's global clipboard.
#assert OS == .MACOS, "The private named-pasteboard lifecycle fixture requires macOS";

#import "Basic";
Writer :: #import "internal/numerics/platform_clipboard"(PASTEBOARD_NAME = "org.cleanroom.numerics.clipboard-test.20261014");
Reader :: #import "internal/numerics/platform_clipboard"(PASTEBOARD_NAME = "org.cleanroom.numerics.clipboard-test.20261014", CLIENT_ID = 1);

main :: () {
    defer Writer.shutdown();
    defer Reader.shutdown();
    ok, message := Writer.write_text("héllo 😀\nsecond line");

    assert(ok, message);
    text: string;
    ok, text, message = Reader.read_text();
    assert(ok, message);
    defer free(text);
    assert(text == "héllo 😀\nsecond line");
    ok, message = Writer.write_text("abc😀");
    assert(ok, message);
    changed: string;
    ok, changed, message = Reader.read_text();
    assert(ok, message);
    defer free(changed);
    assert(changed == "abc😀"); // Reader must synchronize an already-open native handle.
    ok, message = Writer.write_text("");
    assert(ok, message);
    empty: string;
    ok, empty, message = Reader.read_text();
    assert(ok, message);
    defer free(empty);
    assert(empty.count == 0);
    print("native clipboard ok\n");
}
