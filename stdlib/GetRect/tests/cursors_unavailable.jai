// Explicit unsupported-provider contract; this does not claim native cursor creation.
#import "Basic";
Cursor :: #import "internal/numerics/platform_cursors";

#assert OS != .WINDOWS && OS != .MACOS;

main :: () {
    handles, ok, message := Cursor.create();

    assert(!ok);
    assert(message == "Native cursor creation is unavailable on this target; provide the documented cursor callback");
    for handle: handles assert(handle == null);

    ok, message = Cursor.set(null);
    assert(!ok && message == "Cannot set a null native cursor");
    print("cursor provider explicitly unavailable\n");
}
