Repeat :: enum {
    OFF;
    ALL;
    ONE;
}

Queue :: struct {
    items: [..] s64;
    cursor: s64 = -1;
    repeat: Repeat;
    shuffle: bool;
    random: u64 = 0x632be59bd9b4e019;
}

enqueue :: (q: *Queue, track: s64) {
    array_add(*q.items, track);
}

current_track :: (q: *Queue) -> s64 {
    if q.cursor < 0 || q.cursor >= q.items.count return -1;
    return q.items[q.cursor];
}

// Automatic completion respects repeat-one. Explicit Next always advances.
advance :: (q: *Queue, automatic := false) -> bool {
    if !q.items.count return false;
    if automatic && q.repeat == .ONE && q.cursor >= 0 return true;

    next := q.cursor + 1;
    wrapped := next >= q.items.count;

    if wrapped {
        if q.repeat != .ALL return false;
        next = 0;
    }

    if q.shuffle {
        q.random ^= q.random << 13;
        q.random ^= q.random >> 7;
        q.random ^= q.random << 17;
        // Select without replacement from the upcoming suffix. At a repeat-all boundary,
        // exclude the entry just played so distinct queue entries cannot repeat immediately.
        choices := ifx wrapped && q.items.count > 1 then q.items.count - 1 else q.items.count - next;
        chosen := next + cast(s64) (q.random % cast(u64) choices);
        saved := q.items[next];

        q.items[next] = q.items[chosen];
        q.items[chosen] = saved;
    }

    q.cursor = next;

    return true;
}

remove_queued :: (q: *Queue, index: s64) -> bool {
    if index < 0 || index >= q.items.count || index == q.cursor return false;
    for i: index..q.items.count - 2 q.items[i] = q.items[i + 1];

    q.items.count -= 1;
    if index < q.cursor q.cursor -= 1;

    return true;
}
