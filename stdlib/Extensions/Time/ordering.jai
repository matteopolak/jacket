// Comparison, equality and hashing of the value types.

// Orders values (-1/0/1); Zoned and Timestamp use only the instant. Zones order by kind and rule content. Example: `order := compare(a, b)`.
compare :: (a: $T/ .[Date, Time_Of_Day, Date_Time, Timestamp, Signed_Duration, Offset, Time_Zone, Zoned], b: T) -> int {
    #if T == Date {
        c := cmp64(a.year, b.year);
        if c != 0 return c;

        c = cmp64(a.month, b.month);
        if c != 0 return c;

        return cmp64(a.day, b.day);
    } else #if T == Time_Of_Day {
        return cmp64(time_ns(a), time_ns(b));
    } else #if T == Date_Time {
        c := compare(a.date, b.date);
        if c != 0 return c;
        return compare(a.time, b.time);
    } else #if T == Timestamp || T == Signed_Duration {
        c := cmp64(a.seconds, b.seconds);
        if c != 0 return c;
        return cmp64(a.nanoseconds, b.nanoseconds);
    } else #if T == Offset {
        return cmp64(a.seconds, b.seconds);
    } else #if T == Zoned {
        return compare(a.timestamp, b.timestamp);
    } else {
        c := cmp64(cast(s64) a.kind, cast(s64) b.kind);
        if c != 0 return c;
        if a.kind == .FIXED return compare(a.offset, b.offset);

        if a.kind == .RULES && a.rules != null && b.rules != null {
            ra := lookup_rules(a.rules);
            rb := lookup_rules(b.rules);
            if ra == null || rb == null return 0;
            return String.compare(ra.source, rb.source);
        }

        return 0;
    }
}

// Tests value equality consistent with compare and hash; Zoned ignores its zone. Example: `same := equal(a, b)`.
equal :: (a: $T/ .[Date, Time_Of_Day, Date_Time, Timestamp, Signed_Duration, Offset, Time_Zone, Zoned], b: T) -> bool {
    return compare(a, b) == 0;
}

// Hashes semantic fields, never padding or rule pointers; equal values have equal hashes. Example: `key := hash(value)`.
hash :: (value: $T/ .[Date, Time_Of_Day, Date_Time, Timestamp, Signed_Duration, Offset, Time_Zone, Zoned]) -> u32 {
    h: u32 = 2166136261;

    #if T == Date {
        h = hash_part(h, value.year);
        h = hash_part(h, value.month);
        h = hash_part(h, value.day);
    } else #if T == Time_Of_Day {
        h = hash_part(h, time_ns(value));
    } else #if T == Date_Time {
        h = hash_part(h, hash(value.date));
        h = hash_part(h, hash(value.time));
    } else #if T == Timestamp || T == Signed_Duration {
        h = hash_part(h, value.seconds);
        h = hash_part(h, value.nanoseconds);
    } else #if T == Offset {
        h = hash_part(h, value.seconds);
    } else #if T == Zoned {
        return hash(value.timestamp);
    } else {
        h = hash_part(h, cast(s64) value.kind);
        if value.kind == .FIXED h = hash_part(h, value.offset.seconds);

        if value.kind == .RULES && value.rules != null {
            r := lookup_rules(value.rules);
            if r == null return h;
            for c: r.source h = hash_part(h, c);
        }
    }

    return h;
}

#scope_module

hash_part :: (h: u32, v: s64) -> u32 #no_aoc {
    r := h;
    n := cast,no_check(u64) v;

    for 0..7 {
        r = (r ^ cast,trunc(u32) n) * 16777619;
        n >>= 8;
    }

    return r;
}
