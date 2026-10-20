// This private named native pasteboard never accesses the user's global clipboard.
#assert OS == .MACOS, "The private named-pasteboard lifecycle fixture requires macOS";

#import "Basic";
Writer :: #import "internal/numerics/platform_clipboard"(PASTEBOARD_NAME = "org.cleanroom.numerics.clipboard-test.20261014");
Reader :: #import "internal/numerics/platform_clipboard"(PASTEBOARD_NAME = "org.cleanroom.numerics.clipboard-test.20261014", CLIENT_ID = 1);

// The pasteboard server hands a write to other clients asynchronously, so the reader may see
// the previous contents for a moment. Read until the expected text arrives or two seconds pass.
read_expecting :: (expected: string) -> string {
    text: string;

    for 1..200 {
        ok, read, message := Reader.read_text();

        assert(ok, message);

        free(text);
        text = read;
        if text == expected break;

        sleep_milliseconds(10);
    }

    return text;
}

main :: () {
    defer Writer.shutdown();
    defer Reader.shutdown();
    ok, message := Writer.write_text("héllo 😀\nsecond line");

    assert(ok, message);
    text := read_expecting("héllo 😀\nsecond line");

    defer free(text);
    assert(text == "héllo 😀\nsecond line");
    ok, message = Writer.write_text("abc😀");
    assert(ok, message);
    changed := read_expecting("abc😀");

    defer free(changed);
    assert(changed == "abc😀"); // Reader must synchronize an already-open native handle.
    ok, message = Writer.write_text("");
    assert(ok, message);
    empty := read_expecting("");

    defer free(empty);
    assert(empty.count == 0);
    print("native clipboard ok\n");
}
