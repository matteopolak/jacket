#if OS == .MACOS || OS == .LINUX || OS == .ANDROID {
    timespec_to_apollo :: (ts: Clock_Native.timespec) -> Apollo_Time {
        if ts.tv_nsec < 0 || ts.tv_nsec >= 1000000000 return APOLLO_TIME_INVALID;
        duration := add_clock_words(seconds_to_apollo(ts.tv_sec), nanoseconds_to_apollo(ts.tv_nsec));
        return add_clock_words(APOLLO_TIME_FOR_JAN_1_1970, duration);
    }

    to_timespec :: (at: Apollo_Time) -> (ts: Clock_Native.timespec, success: bool) {
        if at.low == APOLLO_TIME_INVALID.low && at.high == APOLLO_TIME_INVALID.high return .{}, false;

        duration := subtract_clock_words(at, APOLLO_TIME_FOR_JAN_1_1970);
        seconds, valid := to_seconds(duration);

        if !valid return .{}, false;

        remainder := subtract_clock_words(duration, seconds_to_apollo(seconds));
        nanos, valid_nanos := to_nanoseconds(remainder);

        if !valid_nanos return .{}, false;

        // timespec uses floor seconds, unlike duration-to-integer truncation.
        if remainder.high < 0 {
            if seconds == -9223372036854775808 return .{}, false;

            seconds -= 1;
            positive_remainder := add_clock_words(remainder, seconds_to_apollo(1));

            nanos, valid_nanos = to_nanoseconds(positive_remainder);
            if !valid_nanos return .{}, false;
        }

        return .{ seconds, nanos }, true;
    }

    current_time_consensus :: () -> Apollo_Time {
        value: Clock_Native.timespec;
        status := Clock_Native.clock_gettime(.REALTIME, *value);

        if status != 0 {
            report_time_failure("realtime clock query failed");
            return APOLLO_TIME_INVALID;
        }

        return timespec_to_apollo(value);
    }

    current_time_monotonic :: () -> Apollo_Time {
        value: Clock_Native.timespec;
        status := Clock_Native.clock_gettime(.MONOTONIC, *value);

        if status != 0 {
            report_time_failure("monotonic clock query failed");
            return APOLLO_TIME_INVALID;
        }

        return add_clock_words(seconds_to_apollo(value.tv_sec), nanoseconds_to_apollo(value.tv_nsec));
    }

    to_calendar :: (at: Apollo_Time, time_zone := Time_Zone.UTC) -> Calendar_Time {
        ts, valid := to_timespec(at);

        if !valid return calendar_failure("Apollo timestamp is outside native calendar range", time_zone);
        if time_zone != .UTC && time_zone != .LOCAL return calendar_failure("invalid calendar time zone", time_zone);

        broken: Clock_Native.tm;
        converted: *Clock_Native.tm;
        seconds: Clock_Native.time_t = ts.tv_sec;

        if time_zone == .UTC converted = Clock_Native.gmtime_r(*seconds, *broken);
        else converted = Clock_Native.localtime_r(*seconds, *broken);

        if !converted return calendar_failure("native calendar conversion failed", time_zone);

        year := cast(s64) broken.tm_year + 1900;

        if year < -2147483648 || year > 2147483647 return calendar_failure("calendar year overflows s32", time_zone);

        return .{ cast(s32) year, cast(s8) broken.tm_mon, cast(s8) (broken.tm_mday - 1),
                  cast(s8) broken.tm_wday, cast(s8) broken.tm_hour, cast(s8) broken.tm_min,
                  cast(s8) broken.tm_sec, cast(s16) (ts.tv_nsec / 1000000), time_zone };
    }

    calendar_to_apollo :: (ct: Calendar_Time) -> Apollo_Time {
        year := cast(s64) ct.year - 1900;
        if year < -2147483648 || year > 2147483647 {
            context.logger("calendar year exceeds native tm range\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
            return .{};
        }

        broken: Clock_Native.tm;
        broken.tm_year = cast(s32) year;
        broken.tm_mon = ct.month_starting_at_0;
        broken.tm_mday = cast(s32) ct.day_of_month_starting_at_0 + 1;
        broken.tm_hour = ct.hour;
        broken.tm_min = ct.minute;
        broken.tm_sec = ct.second;
        broken.tm_isdst = -1;
        seconds: Clock_Native.time_t;
        if ct.time_zone == .UTC seconds = Clock_Native.timegm(*broken);
        else seconds = Clock_Native.mktime(*broken);

        // Reviewed POSIX behavior delegates normalization/DST selection to libc.
        // Its error sentinel collides with the valid instant before the epoch.
        if seconds == -1 return .{};

        return add_clock_words(timespec_to_apollo(.{ seconds, 0 }), milliseconds_to_apollo(ct.millisecond));
    }
}

calendar_failure :: (message: string, time_zone: Time_Zone) -> Calendar_Time {
    context.logger(message, context.logger_data, Log_Info.{ common_flags = .ERROR });
    return .{};
}

#scope_file
#if OS == .MACOS || OS == .LINUX || OS == .ANDROID Clock_Native :: #import "POSIX";

add_clock_words :: (a: Apollo_Time, b: Apollo_Time) -> Apollo_Time #no_aoc {
    low := a.low + b.low;
    high := cast(u64) a.high + cast(u64) b.high;
    if low < a.low high += 1;
    return .{ low, cast(s64) high };
}

subtract_clock_words :: (a: Apollo_Time, b: Apollo_Time) -> Apollo_Time #no_aoc {
    low := a.low - b.low;
    high := cast(u64) a.high - cast(u64) b.high;
    if a.low < b.low high -= 1;
    return .{ low, cast(s64) high };
}
