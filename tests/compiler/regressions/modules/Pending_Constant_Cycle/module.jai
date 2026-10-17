// Module for tests/compiler-regressions/pending-constant-cycle.jai.

#scope_export

IS_DESKTOP :: OS == .MACOS || OS == .LINUX || OS == .WINDOWS;

use_lock :: () -> s32 #compiler {
    #if OS == .WASM {
        return acquire(*lock);
    } else #if IS_DESKTOP {
        return acquire(*lock);
    } else {
        return acquire(*lock);
    }
}

#scope_module

#if OS == .WASM {
    lock: s32;

    acquire :: (l: *s32) -> s32 {
        l.* += 1;
        return l.*;
    }
} else #if IS_DESKTOP {
    lock: s32;

    acquire :: (l: *s32) -> s32 {
        l.* += 1;
        return l.*;
    }
} else {
    lock: s32;

    acquire :: (l: *s32) -> s32 {
        l.* += 1;
        return l.*;
    }
}
