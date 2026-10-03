#if OS == .WINDOWS {
    filetime_to_apollo :: (ft: Filetime_Native.FILETIME) -> Apollo_Time {
        low := cast(u64) ft.dwLowDateTime;
        high := cast(u64) ft.dwHighDateTime;
        result_low, result_high := Filetime_Kernel.ticks_to_words(low | (high << 32));
        return .{ result_low, result_high };
    }

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
    Filetime_Native :: #import "Windows";
    Filetime_Kernel :: #import "internal/time/filetime";
}
