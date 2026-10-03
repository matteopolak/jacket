#import "Basic";
// Library-owned unit may use private implementation; external callers may not.
R :: #import "internal/watching/event-records";

put :: (data: [] u8, index: int, value: u32) {
    for shift: 0..3 data[index + shift] = cast,truncate(u8) (value >> (shift * 8));
}

main :: () {
    linux: [24] u8;

    put(linux, 0, 7);
    put(linux, 4, 0x100);
    put(linux, 8, 42);
    put(linux, 12, 8);
    linux[16] = #char "a";
    record, next, ok := R.inotify_record(linux, 0);

    assert(ok && next == 24 && record.descriptor == 7 && record.flags == 0x100 && record.cookie == 42 && record.name.count == 1 && record.name[0] == #char "a");
    _, _, truncated := R.inotify_record(array_view(linux, 0, 23), 0);

    assert(!truncated);
    for index: 16..23 linux[index] = 1;

    _, _, unterminated := R.inotify_record(linux, 0);

    assert(!unterminated);
    put(linux, 12, 0xffffffff);
    _, _, oversized := R.inotify_record(linux, 0);

    assert(!oversized);
    _, _, negative := R.inotify_record(linux, -1);

    assert(!negative);

    windows: [32] u8;

    put(windows, 0, 16);
    put(windows, 4, 4);
    put(windows, 8, 2);
    windows[12] = #char "a";
    put(windows, 20, 5);
    put(windows, 24, 2);
    windows[28] = #char "b";
    old, cursor, first := R.windows_record(windows, 0);

    assert(first && cursor == 16 && old.flags == 4 && old.name.count == 2 && old.name[0] == #char "a");
    fresh, end, second := R.windows_record(windows, cursor);

    assert(second && end == 32 && fresh.flags == 5 && fresh.name[0] == #char "b");
    put(windows, 0, 15);
    _, _, unaligned := R.windows_record(windows, 0);

    assert(!unaligned);
    put(windows, 0, 8);
    _, _, overlapping := R.windows_record(windows, 0);

    assert(!overlapping);
    put(windows, 0, 16);
    put(windows, 8, 3);
    _, _, odd := R.windows_record(windows, 0);

    assert(!odd);
    _, _, empty := R.windows_record(.[], 0);

    assert(!empty);
    print("ok\n");
}
