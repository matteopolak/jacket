// The allocator each test runs with: it passes every request to the allocator that was in the
// context, and remembers what is live so that a leak or a bad free fails the test.
//
// A tracker is never freed. Memory a test allocated may outlive it (a global array that grew
// during the test keeps the tracker as its allocator), so once its test is over a tracker only
// passes requests on.

#scope_module

Tracker :: struct {
    parent: Allocator;
    active: bool;
    lock: s32;
    live: [] Live_Allocation; // Open addressing; `memory == null` is an empty slot.
    live_count: s64;
    bad_free: bool; // Set once, so a test fails at its first bad free only.
}

// Where an allocation was made, found when it was made: `site` is the first frame outside the standard library (the line
// in the program or test that asked for the memory), and `fallback` the first frame outside this file, for memory the
// standard library allocated on its own.
Live_Allocation :: struct {
    memory: *void;
    size: s64;
    site: Frame;
    fallback: Frame;
}

Frame :: struct {
    info: *Stack_Trace_Procedure_Info;
    line: u32;
}

start_tracking :: (parent: Allocator) -> *Tracker {
    tracker := New(Tracker,, parent);
    tracker.parent = parent;
    tracker.active = true;
    return tracker;
}

tracking_allocator :: (tracker: *Tracker) -> Allocator {
    return .{ tracking_proc, tracker };
}

// Stops tracking and reports what is still live as one failure per site.
finish_tracking :: (tracker: *Tracker, test_location: Source_Code_Location, report_leaks: bool) {
    lock(tracker);
    tracker.active = false;
    unlock(tracker);
    if !report_leaks || !tracker.live_count return;

    // Group the leaks by the line that asked for the memory.
    Site :: struct {
        location: Source_Code_Location;
        procedure: string;
        count: s64;
        bytes: s64;
    }

    sites: [..] Site;
    sites.allocator = temp;
    total_bytes := 0;

    for tracker.live {
        if !it.memory continue;

        total_bytes += it.size;
        location, procedure := site_of(it);
        found := false;

        for *site: sites {
            if site.location.fully_pathed_filename == location.fully_pathed_filename && site.location.line_number == location.line_number {
                site.count += 1;
                site.bytes += it.size;
                found = true;
                break;
            }
        }

        if !found array_add(*sites, .{ location, procedure, 1, it.size });
    }

    for sites {
        where := ifx it.procedure then tprint(" in %", it.procedure) else "";
        location := ifx it.location.fully_pathed_filename then it.location else test_location;
        report_failure(location, tprint("leaked % bytes in % allocation% made here%", it.bytes, it.count, ifx it.count == 1 then "" else "s", where));
    }
}

#scope_file

lock :: (tracker: *Tracker) {
    while true {
        done := compare_and_swap(*tracker.lock, 0, 1);
        if done break;
    }
}

unlock :: (tracker: *Tracker) {
    atomic_exchange(*tracker.lock, 0);
}

// The library's own frames (allocation helpers, `array_add`, this module) are skipped when looking for the line that allocated.
STDLIB_ROOT :: #run stdlib_root();

stdlib_root :: () -> string {
    path := #filepath;
    // .../stdlib/Extensions/Testing/ -> .../stdlib/
    cut := 0;
    seen := 0;
    i := path.count - 2;

    while i >= 0 {
        if path[i] == #char "/" || path[i] == #char "\\" {
            seen += 1;
            if seen == 2 {
                cut = i + 1;
                break;
            }
        }

        i -= 1;
    }

    return string.{ cut, path.data };
}

// The library's own code, not the tests that live in its folders: a leak in a test of the library is reported at the test.
in_stdlib :: (file: string) -> bool {
    if file.count < STDLIB_ROOT.count return false;
    if string.{ STDLIB_ROOT.count, file.data } != STDLIB_ROOT return false;

    below := string.{ file.count - STDLIB_ROOT.count, file.data + STDLIB_ROOT.count };

    return !has_part(below, "/tests/") && !has_part(below, "\\tests\\");
}

has_part :: (text: string, part: string) -> bool {
    for start: 0..text.count - part.count {
        if string.{ part.count, text.data + start } == part return true;
    }

    return false;
}

site_of :: (allocation: Live_Allocation) -> (Source_Code_Location, string) {
    frame := allocation.site;
    if !frame.info frame = allocation.fallback;
    if !frame.info return .{}, "";

    return .{ frame.info.location.fully_pathed_filename, frame.line, 0 }, frame.info.name;
}

hash_pointer :: (memory: *void) -> u64 {
    x := cast(u64) memory;
    x ^= x >> 33;
    x *= 0xff51afd7ed558ccd;
    x ^= x >> 33;

    return x;
}

slot_of :: (tracker: *Tracker, memory: *void) -> s64 {
    mask := cast(u64) tracker.live.count - 1;
    index := hash_pointer(memory) & mask;

    while true {
        slot := *tracker.live[cast(s64) index];
        if slot.memory == memory || slot.memory == null return cast(s64) index;
        index = (index + 1) & mask;
    }

    return -1;
}

grow :: (tracker: *Tracker) {
    old := tracker.live;
    count := ifx old.count then old.count * 2 else 256;
    bytes := count * size_of(Live_Allocation);
    memory := tracker.parent.proc(.ALLOCATE, bytes, 0, null, tracker.parent.data);

    memset(memory, 0, bytes);
    tracker.live = .{ count, memory };
    for old if it.memory tracker.live[slot_of(tracker, it.memory)] = it;
    if old.data tracker.parent.proc(.FREE, 0, old.count * size_of(Live_Allocation), old.data, tracker.parent.data);
}

remember :: (tracker: *Tracker, memory: *void, size: s64) {
    if (tracker.live_count + 1) * 2 > tracker.live.count grow(tracker);

    slot := *tracker.live[slot_of(tracker, memory)];
    slot.memory = memory;
    slot.size = size;
    slot.site = .{};
    slot.fallback = .{};

    frame := context.stack_trace;

    while frame {
        if frame.info && frame.info.location.fully_pathed_filename != #file {
            here := Frame.{ frame.info, frame.line_number };

            if !slot.fallback.info slot.fallback = here;

            if !in_stdlib(frame.info.location.fully_pathed_filename) {
                slot.site = here;
                break;
            }
        }

        frame = frame.next;
    }

    tracker.live_count += 1;
}

// Removes `memory` from the table; false when it was not live.
forget :: (tracker: *Tracker, memory: *void) -> bool {
    if !tracker.live.count return false;

    index := slot_of(tracker, memory);
    if tracker.live[index].memory != memory return false;

    // Backward-shift deletion keeps every probe sequence unbroken.
    mask := tracker.live.count - 1;
    hole := index;
    next := (hole + 1) & mask;

    while tracker.live[next].memory {
        home := cast(s64) (hash_pointer(tracker.live[next].memory) & cast(u64) mask);
        if ((next - home) & mask) >= ((next - hole) & mask) {
            tracker.live[hole] = tracker.live[next];
            hole = next;
        }

        next = (next + 1) & mask;
    }

    tracker.live[hole] = .{};
    tracker.live_count -= 1;

    return true;
}

bad_free :: (tracker: *Tracker, what: string) {
    if tracker.bad_free return;
    tracker.bad_free = true;
    report_failure(caller_frame(), what);
}

// Where the bad free was written: the first frame outside the standard library.
caller_frame :: () -> Source_Code_Location {
    frame := context.stack_trace;
    while frame {
        if frame.info && !in_stdlib(frame.info.location.fully_pathed_filename) {
            return .{ frame.info.location.fully_pathed_filename, frame.line_number, 0 };
        }

        frame = frame.next;
    }

    return running_location();
}

tracking_proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    tracker := cast(*Tracker) allocator_data;
    parent := tracker.parent;

    if !tracker.active return parent.proc(mode, size, old_size, old_memory, parent.data);

    if mode == {
        case .ALLOCATE;
            memory := parent.proc(mode, size, old_size, old_memory, parent.data);
            if memory {
                lock(tracker);
                remember(tracker, memory, size);
                unlock(tracker);
            }

            return memory;
        case .RESIZE;
            if old_memory {
                lock(tracker);
                known := forget(tracker, old_memory);

                unlock(tracker);

                if !known {
                    bad_free(tracker, "resized memory this test's allocator does not own (freed already, or allocated elsewhere)");
                    return parent.proc(mode, size, old_size, old_memory, parent.data);
                }
            }

            memory := parent.proc(mode, size, old_size, old_memory, parent.data);
            if memory {
                lock(tracker);
                remember(tracker, memory, size);
                unlock(tracker);
            }

            return memory;
        case .FREE;
            if !old_memory return null;

            lock(tracker);
            known := forget(tracker, old_memory);

            unlock(tracker);

            if !known {
                // Not passed on: a double free would corrupt the heap and crash the test.
                bad_free(tracker, "freed memory this test's allocator does not own (freed twice, or allocated elsewhere)");
                return null;
            }

            return parent.proc(mode, size, old_size, old_memory, parent.data);
    }

    return parent.proc(mode, size, old_size, old_memory, parent.data);
}
