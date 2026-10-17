// Independent SDK ABI regression: CGWindowLevel.h keys/results are int32_t.
// No unreviewed Jai initializer or mutable globals are required.
#import "Basic";

main :: () {
    #if OS == .MACOS {
        normal := native_level(4);

        assert(normal == 0);
        assert(native_level(5) > normal);
        assert(native_level(1) < native_level(2));
        assert(native_level(14) > native_level(13));
        assert(native_level(3) < normal);
        assert(native_level(20) > native_level(13));
    }
}

#scope_file
#if OS == .MACOS {
    core_graphics :: #library,system "CoreGraphics";
    native_level :: (key: s32) -> s32 #foreign core_graphics "CGWindowLevelForKey";
}
