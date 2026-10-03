#if OS == .WINDOWS {
    to_calendar :: (at: Apollo_Time, time_zone := Time_Zone.UTC) -> Calendar_Time {
        if time_zone != .UTC && time_zone != .LOCAL return calendar_failure("invalid calendar time zone", time_zone);

        ft, valid := to_filetime(at);
        if !valid return calendar_failure("Apollo timestamp is outside Windows calendar range", time_zone);

        utc: Calendar_Windows.SYSTEMTIME;
        if Calendar_Windows.FileTimeToSystemTime(*ft, *utc) == 0 return calendar_failure("Windows calendar conversion failed", time_zone);

        native := utc;

        if time_zone == .LOCAL {
            if utc_to_local(null, *utc, *native) == 0 return calendar_failure("Windows local calendar conversion failed", time_zone);
        }

        return .{ cast(s32) native.wYear, cast(s8) (native.wMonth - 1), cast(s8) (native.wDay - 1),
                  cast(s8) native.wDayOfWeek, cast(s8) native.wHour, cast(s8) native.wMinute,
                  cast(s8) native.wSecond, cast(s16) native.wMilliseconds, time_zone };
    }

    calendar_to_apollo :: (ct: Calendar_Time) -> Apollo_Time {
        if ct.year < 1601 || ct.year > 30827 return windows_calendar_input_failure();
        if ct.month_starting_at_0 < 0 || ct.month_starting_at_0 > 11 return windows_calendar_input_failure();
        if ct.day_of_month_starting_at_0 < 0 || ct.day_of_month_starting_at_0 > 30 return windows_calendar_input_failure();
        if !Calendar_Date.valid_date(ct.year, ct.month_starting_at_0, ct.day_of_month_starting_at_0) return windows_calendar_input_failure();
        if ct.hour < 0 || ct.hour > 23 || ct.minute < 0 || ct.minute > 59 return windows_calendar_input_failure();
        if ct.second < 0 || ct.second > 59 || ct.millisecond < 0 || ct.millisecond > 999 return windows_calendar_input_failure();
        if ct.time_zone != .UTC && ct.time_zone != .LOCAL return windows_calendar_input_failure();

        native: Calendar_Windows.SYSTEMTIME;
        native.wYear = cast(u16) ct.year;
        native.wMonth = cast(u16) (cast(s32) ct.month_starting_at_0 + 1);
        native.wDay = cast(u16) (cast(s32) ct.day_of_month_starting_at_0 + 1);
        native.wHour = cast(u16) ct.hour;
        native.wMinute = cast(u16) ct.minute;
        native.wSecond = cast(u16) ct.second;
        native.wMilliseconds = cast(u16) ct.millisecond;
        ft: Calendar_Windows.FILETIME;
        // Validate the wall-clock fields as a Gregorian date before zone mapping.
        if Calendar_Windows.SystemTimeToFileTime(*native, *ft) == 0 return windows_calendar_input_failure();

        if ct.time_zone == .LOCAL {
            utc: Calendar_Windows.SYSTEMTIME;
            if local_to_utc(null, *native, *utc) == 0 return windows_calendar_input_failure();

            if Calendar_Windows.SystemTimeToFileTime(*utc, *ft) == 0 return windows_calendar_input_failure();
        }

        return filetime_to_apollo(ft);
    }
}

#scope_file
#if OS == .WINDOWS {
    Calendar_Windows :: #import "Windows";
    Calendar_Date :: #import "internal/time/gregorian";

    calendar_kernel32 :: #system_library "kernel32";
    // Official timezoneapi.h ABI; private completion, no new public binding.
    // The optional dynamic-zone pointer is opaque here: every caller passes null.
    utc_to_local :: (zone: *void, utc: *Calendar_Windows.SYSTEMTIME, local: *Calendar_Windows.SYSTEMTIME) -> s32 #foreign calendar_kernel32 "SystemTimeToTzSpecificLocalTimeEx";
    local_to_utc :: (zone: *void, local: *Calendar_Windows.SYSTEMTIME, utc: *Calendar_Windows.SYSTEMTIME) -> s32 #foreign calendar_kernel32 "TzSpecificLocalTimeToSystemTimeEx";

    windows_calendar_input_failure :: () -> Apollo_Time {
        report_time_failure("invalid Windows calendar conversion");
        return APOLLO_TIME_INVALID;
    }
}
