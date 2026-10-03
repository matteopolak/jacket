calendar_to_string :: (ct: Calendar_Time) -> string {
    bytes: [128] u8;
    count: s64;

    calendar_number(bytes.data, *count, cast(s64) ct.day_of_month_starting_at_0 + 1, 1);
    calendar_literal(bytes.data, *count, " ");
    month := clamp(cast(s64) ct.month_starting_at_0, 0, 11);

    calendar_literal(bytes.data, *count, calendar_months[month]);
    calendar_literal(bytes.data, *count, " ");
    calendar_number(bytes.data, *count, ct.year, 1);
    calendar_literal(bytes.data, *count, ", ");
    calendar_clock(bytes.data, *count, ct);

    return copy_temporary_string(.{ count, bytes.data });
}

calendar_to_iso_string :: (ct: Calendar_Time) -> string {
    bytes: [128] u8;
    count: s64;

    calendar_number(bytes.data, *count, ct.year, 1);
    calendar_literal(bytes.data, *count, "-");
    calendar_number(bytes.data, *count, cast(s64) ct.month_starting_at_0 + 1, 2);
    calendar_literal(bytes.data, *count, "-");
    calendar_number(bytes.data, *count, cast(s64) ct.day_of_month_starting_at_0 + 1, 2);
    calendar_literal(bytes.data, *count, "T");
    calendar_clock(bytes.data, *count, ct);
    calendar_literal(bytes.data, *count, ".");
    calendar_number(bytes.data, *count, ct.millisecond, 3);
    if ct.time_zone == .UTC calendar_literal(bytes.data, *count, "Z");

    return copy_temporary_string(.{ count, bytes.data });
}

#scope_file
calendar_months :: string.["January", "February", "March", "April", "May", "June",
                         "July", "August", "September", "October", "November", "December"];

calendar_literal :: (out: *u8, count: *s64, text: string) {
    memcpy(out + count.*, text.data, text.count);
    count.* += text.count;
}

calendar_clock :: (out: *u8, count: *s64, ct: Calendar_Time) {
    calendar_number(out, count, ct.hour, 2);
    calendar_literal(out, count, ":");
    calendar_number(out, count, ct.minute, 2);
    calendar_literal(out, count, ":");
    calendar_number(out, count, ct.second, 2);
}

calendar_number :: (out: *u8, count: *s64, number: s64, width: s64) {
    // Calendar fields fit s32, so negating their promoted values is safe.
    magnitude := number;
    if number < 0 magnitude = -number;

    digits: [20] u8;
    used: s64;

    while true {
        digits[used] = cast(u8) (48 + magnitude % 10);
        used += 1;
        magnitude /= 10;
        if magnitude == 0 break;
    }

    if number < 0 {
        out[count.*] = 45;
        count.* += 1;
        width -= 1;
    }

    while width > used {
        out[count.*] = 48;
        count.* += 1;
        width -= 1;
    }

    while used > 0 {
        used -= 1;
        out[count.*] = digits[used];
        count.* += 1;
    }
}
