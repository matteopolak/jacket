#import "Basic";

libc :: #system_library "libc";
finish :: (status: s32) #foreign libc "exit";

main :: () {
    base := context.allocator;
    caps, name := get_capabilities(base);

    print("caps % heap % name %\n", cast(u64) caps, (caps & .CREATE_HEAP) != 0, name);

    if (caps & .CREATE_HEAP) == 0 {
        print("heap_not_attempted\n");
        #if CPU == .X64 && OS != .IOS finish(1);
        return;
    }

    raw := base.proc(.CREATE_HEAP, 0, 0, null, base.data);

    print("heap_created %\n", raw != null);

    #if CPU == .X64 && OS != .IOS {
        if (caps & .CREATE_HEAP) == 0 || raw == null finish(1);
    } else {
        // Mac ARM64's official negative capability remains a real control.
        #if OS == .MACOS {
            if (caps & .CREATE_HEAP) != 0 || raw != null finish(2);
        }
    }

    if raw base.proc(.DESTROY_HEAP, 0, 0, null, raw);
}
