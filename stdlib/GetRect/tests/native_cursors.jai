#assert OS == .WINDOWS || OS == .MACOS, "Native cursor lifecycle requires the Windows/macOS provider";

#import "Basic";
Cursor :: #import "internal/numerics/platform_cursors";

main :: () {
    handles, ok, message := Cursor.create();
    if !ok print("native cursor creation error: %\n", message, to_standard_error = true);

    assert(ok, message);
    defer Cursor.destroy(handles);
    for handle: handles assert(handle != null);

    print("native cursor lifecycle ok\n");
}
