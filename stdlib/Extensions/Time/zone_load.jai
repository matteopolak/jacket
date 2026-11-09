// Building zones from TZif files and POSIX rules, the cache and the handle registry.

// Returns the allocation-free UTC zone.
//
// ```jai
// zone := time_zone_utc();
// assert(zone.kind == .UTC);
// ```
time_zone_utc :: () -> Time_Zone {
    return .{};
}

// Validates and constructs an allocation-free fixed-offset zone.
//
// ```jai
// o, _ := offset(19800);
// zone := time_zone_fixed(o);
// assert(zone.kind == .FIXED && zone.offset.seconds == 19800);
// ```
time_zone_fixed :: (value: Offset) -> Time_Zone {
    return .{ .FIXED, value, null };
}

// Loads and caches a safe IANA name; directory overrides TZDIR for deterministic fixtures.
//
// ```jai,no_run
// zone, error := time_zone_get("America/New_York");
// assert(error.kind == .NONE);
// defer time_zone_free(*zone);
// instant, _ := timestamp_from_seconds(1720000000);
// assert(offset_at(zone, instant).seconds == -14400);
// ```
time_zone_get :: (name: string, options: Time_Options = .{}, directory: string = "") -> Time_Zone, Time_Error {
    if !valid_options(options) || name.count == 0 || name.count > options.max_text return .{}, .{ .INVALID_VALUE, "invalid zone name", -1 };

    dir := directory;
    if dir.count == 0 dir = Env.environment_variable("TZDIR");

    if dir.count == 0 {
        #if OS != .WINDOWS dir = "/usr/share/zoneinfo";
    }

    // Only names that passed the checks below are ever cached, so a hit needs no re-validation.
    if dir.count != 0 && dir.count <= options.max_text {
        cached, hit := cached_zone(dir, name, options);
        if hit return cached, .{};
    }

    for c: name if !((c >= 65 && c <= 90) || (c >= 97 && c <= 122) || (c >= 48 && c <= 57) || c == 47 || c == 95 || c == 45 || c == 43) return .{}, .{ .INVALID_VALUE, "unsafe zone name", -1 };
    if name[0] == 47 || name[name.count - 1] == 47 return .{}, .{ .INVALID_VALUE, "absolute or empty zone component", -1 };
    for i: 1..name.count - 1 if name[i] == 47 && name[i - 1] == 47 return .{}, .{ .INVALID_VALUE, "empty zone component", -1 };

    if dir.count == 0 return .{}, .{ .UNSUPPORTED, "Windows named zones require TZDIR", -1 };
    if dir.count > options.max_text return .{}, .{ .LIMIT, "zone directory limit", -1 };

    path := tprint("%/%", dir, name);

    // Cache storage must outlive a caller's arena or counting allocator.
    push_allocator(context.default_allocator.proc, context.default_allocator.data);
    canonical, exists := Files.get_absolute_path(path);

    defer free(canonical);
    base, base_ok := Files.get_absolute_path(dir);

    defer free(base);
    if !exists || !base_ok return .{}, .{ .NOT_FOUND, "cannot resolve zone path", -1 };
    if !begins_with(canonical, tprint("%/", base)) return .{}, .{ .INVALID_VALUE, "zone path escapes directory", -1 };

    bytes, e := read_zone_file(canonical, options);
    if e.kind != .NONE return .{}, e;

    defer free(bytes.data);
    z, ze := time_zone_from_tzif(name, bytes, options);
    if ze.kind != .NONE return .{}, ze;

    r := cast(*Zone_Rules) z.rules;
    r.path = copy_string(path);
    cache_zone(z);

    return z, .{};
}

// Uses TZ, otherwise /etc/localtime; Windows without TZ returns UNSUPPORTED.
//
// ```jai,no_run
// zone, error := time_zone_system();
// if error.kind == .NONE {
//     defer time_zone_free(*zone);
//     wall := to_civil(zone, timestamp_now());
//     print("local hour: %\n", wall.time.hour);
// }
// ```
time_zone_system :: (options: Time_Options = .{}) -> Time_Zone, Time_Error {
    text := Env.environment_variable("TZ");
    if text.count == 0 && Env.has_environment_variable("TZ") return time_zone_utc(), .{};

    if text.count != 0 {
        if text[0] == 58 return time_zone_get(slice(text, 1, text.count - 1), options);
        zone, e := time_zone_from_posix(text, options);
        if e.kind == .NONE return zone, e;
        return time_zone_get(text, options);
    }

    #if OS == .WINDOWS return .{}, .{ .UNSUPPORTED, "Windows system zone without TZ is not supported yet", -1 };
    else {
        bytes, e := read_zone_file("/etc/localtime", options);
        if e.kind != .NONE return .{}, e;

        defer free(bytes.data);
        name: string;
        canonical, ok := Files.get_absolute_path("/etc/localtime");

        defer free(canonical);

        if ok {
            at := find_index_from_left(canonical, "/zoneinfo/");
            if at >= 0 name = slice(canonical, at + 10, canonical.count - at - 10);
        }

        return time_zone_from_tzif(name, bytes, options);
    }
}

// Copies validated TZif v1-v4 rules and POSIX footer within bounded storage.
//
// ```jai
// bytes := u8.[
//     0x54, 0x5a, 0x69, 0x66, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 4,
//     0, 0, 0x1c, 0x20, 0, 0, 0x4d, 0x59, 0x54, 0, // one type: +02:00 "MYT"
// ];
// zone, error := time_zone_from_tzif("Example/Zone", bytes);
// assert(error.kind == .NONE);
// defer time_zone_free(*zone);
// instant, _ := timestamp_from_seconds(0);
// assert(offset_at(zone, instant).seconds == 7200);
// ```
time_zone_from_tzif :: (name: string, bytes: [] u8, options: Time_Options = .{}) -> Time_Zone, Time_Error {
    if !valid_options(options) || bytes.count > options.max_tzif_bytes || name.count > options.max_text return .{}, .{ .LIMIT, "zone data limit", 0 };

    r := Binary_Reader.{ bytes = bytes };
    h := tzif_header(*r, options);
    width: s64 = 4;

    if r.error.kind != .NONE return .{}, r.error;

    if h.version != 0 {
        r.at += block_size(h, 4);
        previous := h.version;

        h = tzif_header(*r, options);
        width = 8;
        if h.version != previous binary_fail(*r, "TZif section version mismatch");
    }

    if r.error.kind != .NONE return .{}, r.error;

    start := r.at;
    size := block_size(h, width);

    if size > bytes.count - start return .{}, .{ .SYNTAX, "truncated TZif block", start };
    if h.leaps != 0 return .{}, .{ .UNSUPPORTED, "leap-second TZif is outside scope", start };

    ta := start;
    ia := ta + h.times * width;
    types := ia + h.times;
    ab := types + h.types * 6;
    last: s64 = MIN_I64;

    for i: 0..h.times - 1 {
        value := be_signed(bytes, ta + i * width, width);
        if (i > 0 && value <= last) || bytes[ia + i] >= h.types return .{}, .{ .INVALID_VALUE, "invalid TZif transition", ta + i * width };
        last = value;
    }

    for i: 0..h.types - 1 {
        at := types + i * 6;
        off := be_signed(bytes, at, 4);
        index: s64 = bytes[at + 5];

        if off < -93599 || off > 93599 || bytes[at + 4] > 1 || index >= h.chars return .{}, .{ .INVALID_VALUE, "invalid TZif type", at };

        end := index;
        while end < h.chars && bytes[ab + end] != 0 end += 1;
        if end == h.chars return .{}, .{ .INVALID_VALUE, "unterminated TZif abbreviation", ab + index };
    }

    flag_at := ab + h.chars;

    for i: 0..h.standard + h.utc - 1 {
        if bytes[flag_at + i] > 1 return .{}, .{ .INVALID_VALUE, "invalid TZif indicator", flag_at + i };
        if i >= h.standard && bytes[flag_at + i] == 1 && (h.standard == 0 || bytes[flag_at + i - h.standard] == 0) return .{}, .{ .INVALID_VALUE, "UT indicator requires standard indicator", flag_at + i };
    }

    r.at = start + size;
    footer: string;

    if width == 8 {
        if r.at >= bytes.count || bytes[r.at] != 10 return .{}, .{ .SYNTAX, "missing TZif footer", r.at };

        r.at += 1;
        begin := r.at;

        while r.at < bytes.count && bytes[r.at] != 10 r.at += 1;
        if r.at >= bytes.count || r.at + 1 != bytes.count return .{}, .{ .SYNTAX, "invalid TZif footer terminator", r.at };

        footer = string.{ r.at - begin, bytes.data + begin };
        if footer.count > options.max_text return .{}, .{ .LIMIT, "TZif footer limit", begin };

        if footer.count != 0 {
            text := Text_Reader.{ text = footer };

            read_posix(*text);

            if text.error.kind != .NONE {
                text.error.offset += begin;
                return .{}, text.error;
            }
        }
    } else if r.at != bytes.count return .{}, .{ .SYNTAX, "trailing TZif bytes", r.at };

    owner := new_rules(name, string.{ bytes.count, bytes.data });
    owner.width = width;
    owner.count = h.times;
    owner.types = h.types;
    owner.ta = ta;
    owner.ia = ia;
    owner.type_at = types;
    owner.ab = ab;
    owner.chars = h.chars;

    if footer.count != 0 {
        text := Text_Reader.{ text = string.{ footer.count, owner.source.data + (footer.data - bytes.data) } };
        owner.footer = read_posix(*text);
    }

    return .{ .RULES, .{}, owner }, .{};
}

// Parses POSIX TZ rules including M/J/Julian forms and extended transition times.
//
// ```jai
// zone, error := time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");
// assert(error.kind == .NONE);
// defer time_zone_free(*zone);
// summer, _ := parse(Timestamp, "2024-07-01T00:00:00Z");
// assert(offset_at(zone, summer).seconds == -14400);
// ```
time_zone_from_posix :: (text: string, options: Time_Options = .{}) -> Time_Zone, Time_Error {
    if !valid_options(options) || text.count > options.max_text return .{}, .{ .LIMIT, "zone text limit", 0 };

    r := Text_Reader.{ text = text };
    rules := read_posix(*r);

    if r.error.kind != .NONE return .{}, r.error;

    owner := new_rules(name = "", source = text);
    owner.footer = rules;
    // Reparse so abbreviation views borrow the owned copy.
    r = .{ text = owner.source };
    owner.footer = read_posix(*r);

    return .{ .RULES, .{}, owner }, .{};
}

// Acquires an independent owner; UTC and fixed zones need no reference count.
//
// ```jai
// zone, _ := time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");
// owned := time_zone_retain(zone);
// time_zone_free(*zone);
// now_instant, _ := timestamp_from_seconds(1720000000);
// assert(offset_at(owned, now_instant).seconds == -14400);
// time_zone_free(*owned);
// ```
time_zone_retain :: (value: Time_Zone) -> Time_Zone {
    if value.kind == .RULES && value.rules != null {
        r := lookup_rules(value.rules);
        if r == null return .{};
        Atomics.atomic_increment(*r.refs);
    }

    return value;
}

// Releases one owned reference and resets the handle to UTC; borrowed copies become invalid.
//
// ```jai
// zone, _ := time_zone_from_posix("EST5EDT,M3.2.0,M11.1.0");
// time_zone_free(*zone);
// assert(zone.kind == .UTC);
// ```
time_zone_free :: (value: *Time_Zone) {
    if value.kind == .RULES && value.rules != null {
        r := lookup_rules(value.rules);
        if r == null {
            value.* = .{};
            return;
        }

        if Atomics.atomic_decrement(*r.refs) {
            unregister_rules(r);
            push_allocator(r.allocator.proc, r.allocator.data);
            free(r.path);
            free(r.name);
            free(r.source);
            free(r);
        }
    }

    value.* = .{};
}

#scope_module

Atomics :: #import "Atomics";

Files :: #import "File";

Env :: #import "Extensions/Environment";

String :: #import "String";

Zone_Rule :: struct {
    kind: u8;
    a: s64;
    b: s64;
    c: s64;
    seconds: s64 = 7200;
}

Posix_Rules :: struct {
    present: bool;
    dst: bool;
    standard: s64;
    daylight: s64;
    standard_name: string;
    daylight_name: string;
    start: Zone_Rule;
    end: Zone_Rule;
}

Zone_Rules :: struct {
    refs: s64 = 1;
    allocator: Allocator;
    name: string;
    path: string;
    source: string;
    count: s64;
    types: s64;
    width: s64;
    ta: s64;
    ia: s64;
    type_at: s64;
    ab: s64;
    chars: s64;
    footer: Posix_Rules;
}

// Zones loaded by time_zone_get, each holding one reference, replaced in turn once full.
// Only the first zone_cache_used entries can be set; zone_cache_lock guards all of these.
zone_cache: [64] Time_Zone;

zone_cache_slot: s64;

zone_cache_used: s64;

zone_cache_lock: s32;

// Whether path is `dir/name`, without building the text.
path_is :: (path: string, dir: string, name: string) -> bool {
    if path.count != dir.count + 1 + name.count || path[dir.count] != #char "/" return false;
    return begins_with(path, dir) && string.{ name.count, path.data + dir.count + 1 } == name;
}

// Retains the cached zone for `dir/name` when it was loaded within `options`' limits.
cached_zone :: (dir: string, name: string, options: Time_Options) -> Time_Zone, bool {
    while !compare_and_swap(*zone_cache_lock, cast(s32) 0, cast(s32) 1) {}

    defer Atomics.atomic_write(*zone_cache_lock, cast(s32) 0);

    for 0..zone_cache_used - 1 {
        entry := zone_cache[it];
        if entry.rules == null continue;

        // The cache owns a reference, so the rules are live and need no registry check.
        r := cast(*Zone_Rules) entry.rules;

        if path_is(r.path, dir, name) && zone_within_limits(r, options) {
            Atomics.atomic_increment(*r.refs);
            return entry, true;
        }
    }

    return .{}, false;
}

// Keeps a reference to a freshly loaded zone, reusing the slot of the same path or else the oldest.
cache_zone :: (zone: Time_Zone) {
    while !compare_and_swap(*zone_cache_lock, cast(s32) 0, cast(s32) 1) {}

    defer Atomics.atomic_write(*zone_cache_lock, cast(s32) 0);

    path := (cast(*Zone_Rules) zone.rules).path;
    slot := zone_cache_slot;
    found := false;

    for 0..zone_cache_used - 1 {
        entry := zone_cache[it];

        if entry.rules != null && (cast(*Zone_Rules) entry.rules).path == path {
            slot = it;
            found = true;
            break;
        }
    }

    time_zone_free(*zone_cache[slot]);
    zone_cache[slot] = time_zone_retain(zone);

    if !found zone_cache_slot = (zone_cache_slot + 1) % zone_cache.count;
    if slot >= zone_cache_used zone_cache_used = slot + 1;
}

new_rules :: (name: string, source: string) -> *Zone_Rules {
    r := New(Zone_Rules);
    r.allocator = context.allocator;
    r.name = copy_string(name);
    r.source = copy_string(source);
    while !compare_and_swap(*registry_lock, cast(s32) 0, cast(s32) 1) {}

    {
        push_allocator(context.default_allocator.proc, context.default_allocator.data);
        Registry.table_add(*registered_rules, cast(*void) r, r);
    }

    Atomics.atomic_write(*registry_lock, cast(s32) 0);

    return r;
}

valid_options :: (o: Time_Options) -> bool {
    return o.max_text >= 0 && o.max_tzif_bytes >= 0 && o.max_transitions >= 0 && o.max_types >= 0 && o.max_abbreviation_bytes >= 0;
}

zone_within_limits :: (r: *Zone_Rules, o: Time_Options) -> bool {
    return r.source.count <= o.max_tzif_bytes && r.count <= o.max_transitions && r.types <= o.max_types && r.chars <= o.max_abbreviation_bytes;
}

read_zone_file :: (path: string, o: Time_Options) -> [] u8, Time_Error {
    if !valid_options(o) return .[], .{ .LIMIT, "negative zone limit", -1 };

    file, ok := Files.file_open(path);
    if !ok return .[], .{ .NOT_FOUND, "cannot open zone file", -1 };

    defer Files.file_close(*file);
    length, known := Files.file_length(file);
    if !known return .[], .{ .IO, "cannot size zone file", -1 };
    if length < 0 || length > o.max_tzif_bytes return .[], .{ .LIMIT, "zone file size limit", -1 };

    bytes := NewArray(length, u8);
    success, n := Files.file_read(file, bytes.data, length);

    if !success || n != length {
        free(bytes.data);
        return .[], .{ .IO, "cannot read zone file", -1 };
    }

    return bytes, .{};
}

Binary_Reader :: struct {
    bytes: [] u8;
    at: s64;
    error: Time_Error;
}

TZif_Header :: struct {
    version: u8;
    utc: s64;
    standard: s64;
    leaps: s64;
    times: s64;
    types: s64;
    chars: s64;
}

binary_fail :: (r: *Binary_Reader, message: string) {
    if r.error.kind == .NONE r.error = .{ .SYNTAX, message, r.at };
}

be_u32 :: (bytes: $T, at: s64) -> u32 {
    a: u32 = bytes[at];
    b: u32 = bytes[at + 1];
    c: u32 = bytes[at + 2];
    d: u32 = bytes[at + 3];

    return (a << 24) | (b << 16) | (c << 8) | d;
}

be_signed :: (bytes: $T, at: s64, width: s64) -> s64 {
    high := be_u32(bytes, at);
    if width == 4 return cast(s64) cast,no_check(s32) high;

    wide: u64 = high;
    wide = (wide << 32) | be_u32(bytes, at + 4);

    return cast,no_check(s64) wide;
}

tzif_header :: (r: *Binary_Reader, o: Time_Options) -> TZif_Header {
    h: TZif_Header;

    if r.at < 0 || r.at > r.bytes.count || r.bytes.count - r.at < 44 {
        binary_fail(r, "truncated TZif header");
        return h;
    }

    at := r.at;

    r.at += 44;
    if r.bytes[at] != 84 || r.bytes[at + 1] != 90 || r.bytes[at + 2] != 105 || r.bytes[at + 3] != 102 {
        binary_fail(r, "invalid TZif magic");
        return h;
    }

    h.version = r.bytes[at + 4];
    if h.version != 0 && (h.version < 50 || h.version > 52) binary_fail(r, "unsupported TZif version");
    for i: 5..19 if r.bytes[at + i] != 0 binary_fail(r, "nonzero TZif reserved byte");

    h.utc = be_signed(r.bytes, at + 20, 4);
    h.standard = be_signed(r.bytes, at + 24, 4);
    h.leaps = be_signed(r.bytes, at + 28, 4);
    h.times = be_signed(r.bytes, at + 32, 4);
    h.types = be_signed(r.bytes, at + 36, 4);
    h.chars = be_signed(r.bytes, at + 40, 4);
    if h.times < 0 || h.times > o.max_transitions || h.types < 1 || h.types > 256 || h.types > o.max_types || h.chars < 1 || h.chars > o.max_abbreviation_bytes || h.leaps < 0 || h.leaps > o.max_transitions || (h.utc != 0 && h.utc != h.types) || (h.standard != 0 && h.standard != h.types) binary_fail(r, "invalid TZif counts");

    return h;
}

block_size :: (h: TZif_Header, w: s64) -> s64 {
    return h.times * (w + 1) + h.types * 6 + h.chars + h.leaps * (w + 4) + h.standard + h.utc;
}

read_variable :: (r: *Text_Reader, max_digits: s64 = 3) -> s64 {
    n: s64;
    count: s64;

    while peek(r) >= 48 && peek(r) <= 57 && count < max_digits {
        n = n * 10 + peek(r) - 48;
        r.at += 1;
        count += 1;
    }

    if count == 0 text_fail(r);

    return n;
}

posix_name :: (r: *Text_Reader) -> string {
    quoted := peek(r) == 60;
    if quoted r.at += 1;

    begin := r.at;

    while r.at < r.text.count {
        c := peek(r);
        if (c >= 65 && c <= 90) || (c >= 97 && c <= 122) || (quoted && ((c >= 48 && c <= 57) || c == 43 || c == 45)) r.at += 1;
        else break;
    }

    name := string.{ r.at - begin, r.text.data + begin };
    if name.count < 3 text_fail(r);
    if quoted expect_byte(r, 62);

    return name;
}

posix_time :: (r: *Text_Reader, limit: s64) -> s64 {
    sign: s64 = 1;

    if peek(r) == 45 || peek(r) == 43 {
        if peek(r) == 45 sign = -1;
        r.at += 1;
    }

    h := read_variable(r);
    m: s64;
    sec: s64;

    if peek(r) == 58 {
        r.at += 1;
        m = read_number(r, 2);

        if peek(r) == 58 {
            r.at += 1;
            sec = read_number(r, 2);
        }
    }

    if h > limit || m > 59 || sec > 59 text_fail(r, .INVALID_VALUE, "POSIX time out of range");

    return sign * (h * 3600 + m * 60 + sec);
}

posix_rule :: (r: *Text_Reader) -> Zone_Rule {
    rule: Zone_Rule;

    if peek(r) == 77 {
        r.at += 1;
        rule.kind = 2;
        rule.a = read_variable(r);
        expect_byte(r, 46);
        rule.b = read_variable(r);
        expect_byte(r, 46);
        rule.c = read_variable(r);
        if rule.a < 1 || rule.a > 12 || rule.b < 1 || rule.b > 5 || rule.c > 6 text_fail(r, .INVALID_VALUE, "invalid M rule");
    } else {
        if peek(r) == 74 {
            r.at += 1;
            rule.kind = 0;
        } else rule.kind = 1;

        rule.a = read_variable(r);
        if (rule.kind == 0 && (rule.a < 1 || rule.a > 365)) || (rule.kind == 1 && rule.a > 365) text_fail(r, .INVALID_VALUE, "invalid Julian rule");
    }

    if peek(r) == 47 {
        r.at += 1;
        rule.seconds = posix_time(r, 167);
    }

    return rule;
}

read_posix :: (r: *Text_Reader) -> Posix_Rules {
    p: Posix_Rules;
    p.present = true;
    p.standard_name = posix_name(r);
    p.standard = -posix_time(r, 25);
    if p.standard < -93599 || p.standard > 93599 text_fail(r, .OUT_OF_RANGE, "POSIX offset out of range");

    if r.at < r.text.count && r.error.kind == .NONE {
        p.dst = true;
        p.daylight_name = posix_name(r);
        p.daylight = p.standard + 3600;
        if peek(r) != 44 p.daylight = -posix_time(r, 25);
        if p.daylight < -93599 || p.daylight > 93599 text_fail(r, .OUT_OF_RANGE, "POSIX DST offset out of range");

        expect_byte(r, 44);
        p.start = posix_rule(r);
        expect_byte(r, 44);
        p.end = posix_rule(r);
    }

    if r.at != r.text.count text_fail(r);

    return p;
}

Registry :: #import "Hash_Table";

registered_rules: Registry.Table(*void, *Zone_Rules);

registry_lock: s32;

// Handles lookup_rules found registered, indexed by a few address bits. A slot holds null or a
// registered handle: unregister_rules clears it under registry_lock before the rules go, so a
// matching slot proves the handle is live without taking the lock.
validated_rules: [16] *void;

validated_slot :: (handle: *void) -> s64 {
    address := cast(u64) handle;
    return cast(s64) ((address >> 4) & 15);
}

// Validate opaque handles before dereferencing: transparent literals may contain arbitrary bits.
lookup_rules :: (handle: *void) -> *Zone_Rules {
    slot := validated_slot(handle);
    if Atomics.atomic_read(*validated_rules[slot]) == handle return cast(*Zone_Rules) handle;

    while !compare_and_swap(*registry_lock, cast(s32) 0, cast(s32) 1) {}

    _, r := Registry.table_find(*registered_rules, handle);
    if r != null Atomics.atomic_write(*validated_rules[slot], handle);

    Atomics.atomic_write(*registry_lock, cast(s32) 0);

    return r;
}

unregister_rules :: (r: *Zone_Rules) {
    while !compare_and_swap(*registry_lock, cast(s32) 0, cast(s32) 1) {}

    slot := validated_slot(cast(*void) r);
    if validated_rules[slot] == cast(*void) r Atomics.atomic_write(*validated_rules[slot], cast(*void) null);

    Registry.table_remove(*registered_rules, cast(*void) r);
    if registered_rules.count == 0 Registry.deinit(*registered_rules);

    Atomics.atomic_write(*registry_lock, cast(s32) 0);
}
