#if OS == .WINDOWS {
    // Converts a Windows `FILETIME` (100 ns ticks since 1601) to an `Apollo_Time`. Windows only. Every `FILETIME` is
    // representable.
    filetime_to_apollo :: (ft: Filetime_Native.FILETIME) -> Apollo_Time {
        low := cast(u64) ft.dwLowDateTime;
        high := cast(u64) ft.dwHighDateTime;
        result_low, result_high := Filetime_Kernel.ticks_to_words(low | (high << 32));
        return .{ result_low, result_high };
    }

    // Converts an `Apollo_Time` to a Windows `FILETIME`, with `false` and a zero value when the time lies before 1601 or beyond
    // what a `FILETIME` can hold. Windows only. Times are cut to a multiple of 100 ns.
    to_filetime :: (at: Apollo_Time) -> (ft: Filetime_Native.FILETIME, success: bool) {
        ticks, valid := Filetime_Kernel.words_to_ticks(at.low, at.high);
        if !valid return .{}, false;

        result: Filetime_Native.FILETIME;
        result.dwLowDateTime = cast(u32) (ticks & 0xffffffff);
        result.dwHighDateTime = cast(u32) (ticks >> 32);

        return result, true;
    }
}

#scope_file
#if OS == .WINDOWS {
    Filetime_Native :: #import "internal/bindings/windows-api";
    Filetime_Kernel :: #import "internal/time/filetime";
}
