// The two matchers that run a Program directly and report captures: a Pike VM, which runs
// every thread in lockstep and works on any input, and a backtracker that remembers which
// (instruction, position) pairs it has tried, which is faster on short spans. Both take
// O(instructions * text) time at most and agree with each other on every match.

// Whether `look` holds at `pos`. The whole text is context, so a search that starts mid-text
// still sees the byte before it.
look_holds :: (look: Look, text: string, pos: int) -> bool {
    if look == {
        case .START_TEXT; return pos == 0;
        case .END_TEXT; return pos == text.count;
        case .START_LINE; return pos == 0 || text[pos - 1] == #char "\n";
        case .END_LINE; return pos == text.count || text[pos] == #char "\n";
        case .START_LINE_CRLF;
            if pos == 0 || text[pos - 1] == #char "\n" return true;

            return text[pos - 1] == #char "\r" && (pos == text.count || text[pos] != #char "\n");
        case .END_LINE_CRLF;
            if pos == text.count || text[pos] == #char "\r" return true;

            return text[pos] == #char "\n" && (pos == 0 || text[pos - 1] != #char "\r");
    }

    before, after: bool;

    if look >= .WORD_ASCII {
        before = pos > 0 && is_word_byte(text[pos - 1]);
        after = pos < text.count && is_word_byte(text[pos]);
    } else {
        before = word_char_before(text, pos);
        after = word_char_after(text, pos);
    }

    if look == {
        case .WORD; #through;
        case .WORD_ASCII; return before != after;
        case .NOT_WORD; #through;
        case .NOT_WORD_ASCII; return before == after;
        case .WORD_START; #through;
        case .WORD_START_ASCII; return !before && after;
        case .WORD_END; #through;
        case .WORD_END_ASCII; return before && !after;
        case .WORD_START_HALF; #through;
        case .WORD_START_HALF_ASCII; return !before;
        case .WORD_END_HALF; #through;
        case .WORD_END_HALF_ASCII; return !after;
    }

    return false;
}

is_word_byte :: inline (b: u8) -> bool {
    return (b >= #char "a" && b <= #char "z") || (b >= #char "A" && b <= #char "Z") || (b >= #char "0" && b <= #char "9") || b == #char "_";
}

word_char_after :: (text: string, pos: int) -> bool {
    if pos >= text.count return false;
    if text[pos] < 0x80 return is_word_byte(text[pos]);

    c, size := decode_utf8(text, pos);
    if size == 1 return false; // not valid UTF-8

    return is_word_char(c);
}

word_char_before :: (text: string, pos: int) -> bool {
    if pos <= 0 return false;
    if text[pos - 1] < 0x80 return is_word_byte(text[pos - 1]);

    // Back up to the lead byte of the sequence that ends here.
    start := pos - 1;
    while start > 0 && pos - start < 4 && (text[start] & 0xC0) == 0x80 start -= 1;

    c, size := decode_utf8(text, start);
    if start + size != pos || size == 1 return false;

    return is_word_char(c);
}

is_word_char :: (c: u32) -> bool {
    if !word_table_ready {
        word_table = unicode_perl_word();
        word_table_ready = true;
    }

    lo, hi := 0, word_table.count - 1;
    while lo <= hi {
        mid := (lo + hi) / 2;
        r := word_table[mid];

        if c < r.lo hi = mid - 1;
        else if c > r.hi lo = mid + 1;
        else return true;
    }

    return false;
}

// Runs `program` over text[start .. end] (looks still see all of `text`). Anchored searches only
// try `start`. Fills `slots` (the first slots.count of them, -1 for groups that did not take
// part: pass two to find only where the match is) and
// returns whether anything matched; the match is the leftmost one, and among those, the one
// the pattern prefers.
pike_search :: (program: *Program, text: string, start: int, end: int, anchored: bool, slots: [] int) -> bool {
    // Everything here is scratch: it all goes back to temporary storage on return.
    auto_release_temp();
    vm: Pike;
    vm.program = program;
    vm.text = text;
    vm.slot_count = min(slots.count, program.slot_count);
    n := program.insts.count;

    init_threads(*vm.current, n, vm.slot_count);
    init_threads(*vm.next, n, vm.slot_count);
    scratch := NewArray(vm.slot_count, int,, temp);

    vm.work = NewArray(vm.slot_count, int,, temp);
    vm.stack.allocator = temp;
    matched := false;
    pos := start;

    while true {
        if !matched && (!anchored || pos == start) {
            for *scratch it.* = -1;
            add_thread(*vm, *vm.current, program.start, pos, scratch);
        }

        if vm.current.count == 0 && (matched || anchored) break;

        // Threads in priority order. A match cuts off every thread after it: they could only
        // produce matches the pattern likes less.
        for i: 0..vm.current.count - 1 {
            pc := vm.current.dense[i];
            inst := *program.insts[pc];
            thread_slots := thread_slots(*vm.current, pc);

            if inst.kind == .MATCH {
                for 0..vm.slot_count - 1 slots[it] = thread_slots[it];
                matched = true;
                break;
            }

            if inst.kind == .RANGES && pos < end {
                b := text[pos];

                for t: 0..inst.alt - 1 {
                    tr := *program.transitions[inst.next + t];
                    if b >= tr.lo && b <= tr.hi add_thread(*vm, *vm.next, tr.next, pos + 1, thread_slots);
                }
            }
        }

        if pos >= end break;

        pos += 1;
        vm.current, vm.next = vm.next, vm.current;
        vm.next.count = 0;
    }

    return matched;
}

// The first match at exactly `start` within text[start .. end], like an anchored pike_search,
// by depth-first search in priority order. Each (instruction, position) is tried once, so this
// is linear too, but it needs instructions * (end - start + 1) bits of memory: use it for short
// spans only (backtrack_fits).
backtrack_search :: (program: *Program, text: string, start: int, end: int, slots: [] int) -> bool {
    width := end - start + 1;
    words := (program.insts.count * width + 63) / 64;
    visited := NewArray(words, u64);

    defer array_free(visited);
    for *slots it.* = -1;

    stack: [..] Frame;

    defer array_free(stack);
    array_add(*stack, .{ pc = program.start, pos = start, restore = -1 });

    while stack.count {
        frame := pop(*stack);
        if frame.restore >= 0 {
            slots[frame.restore] = frame.pos;
            continue;
        }

        pc := frame.pc;
        pos := frame.pos;

        while true {
            bit := cast(int) pc * width + (pos - start);
            if visited[bit >> 6] & (cast(u64) 1 << cast(u64) (bit & 63)) break;

            visited[bit >> 6] |= cast(u64) 1 << cast(u64) (bit & 63);
            inst := *program.insts[pc];
            if inst.kind == {
                case .MATCH;
                    return true;
                case .RANGES;
                    if pos >= end break;

                    b := text[pos];
                    // Overlapping transitions are alternatives: take the first that fits and
                    // stack the rest so that they pop in order.
                    first: u32 = 0xFFFF_FFFF;

                    for < t: 0..inst.alt - 1 {
                        tr := *program.transitions[inst.next + t];
                        if b < tr.lo || b > tr.hi continue;
                        if first != 0xFFFF_FFFF array_add(*stack, .{ pc = first, pos = pos + 1, restore = -1 });
                        first = tr.next;
                    }

                    if first == 0xFFFF_FFFF break;

                    pc = first;
                    pos += 1;
                case .SPLIT;
                    array_add(*stack, .{ pc = inst.alt, pos = pos, restore = -1 });
                    pc = inst.next;
                case .SAVE;
                    array_add(*stack, .{ pos = slots[inst.alt], restore = cast(int) inst.alt });
                    slots[inst.alt] = pos;
                    pc = inst.next;
                case .LOOK;
                    if !look_holds(inst.look, text, pos) break;

                    pc = inst.next;
                case .FAIL;
                    break;
            }
        }
    }

    return false;
}

backtrack_fits :: (program: *Program, span: int) -> bool {
    return program.insts.count * (span + 1) <= 256 * 1024 * 8;
}

#scope_file

word_table: [..] Char_Range;
word_table_ready := false;

Frame :: struct {
    pc: u32;
    pos: int;
    restore: int; // >= 0: put `pos` back into this slot
}

Pike :: struct {
    program: *Program;
    text: string;
    slot_count: int;
    current: Threads;
    next: Threads;
    stack: [..] Pike_Frame;
    work: [] int; // add_thread's slots
}

Pike_Frame :: struct {
    pc: u32;
    restore: int; // >= 0: put `value` back into this slot instead of exploring pc
    value: int;
}

// A sparse set of instructions with a slot array per member.
Threads :: struct {
    dense: [] u32;
    sparse: [] u32;
    count: int;
    slots: [] int;
    slot_count: int;
}

init_threads :: (t: *Threads, n: int, slot_count: int) {
    t.dense = NewArray(n, u32,, temp);
    t.sparse = NewArray(n, u32,, temp);
    t.slots = NewArray(n * slot_count, int, initialized = false,, temp);
    t.slot_count = slot_count;
}

contains :: inline (t: *Threads, pc: u32) -> bool {
    i := t.sparse[pc];
    return i < t.count && t.dense[i] == pc;
}

thread_slots :: inline (t: *Threads, pc: u32) -> [] int {
    view: [] int;
    view.data = t.slots.data + pc * cast(u32) t.slot_count;
    view.count = t.slot_count;
    return view;
}

// Follows SPLIT, SAVE and LOOK from `pc` and adds every instruction that waits for a byte (or
// matches) to `list`, in priority order, each with the slots it was reached with.
add_thread :: (vm: *Pike, list: *Threads, pc: u32, pos: int, slots_in: [] int) {
    slots := vm.work;
    for slots_in slots[it_index] = it;

    stack := *vm.stack;
    stack.count = 0;
    array_add(stack, .{ pc = pc, restore = -1 });

    while stack.count {
        frame := pop(stack);
        if frame.restore >= 0 {
            slots[frame.restore] = frame.value;
            continue;
        }

        at := frame.pc;

        while true {
            if contains(list, at) break;

            list.sparse[at] = cast(u32) list.count;
            list.dense[list.count] = at;
            list.count += 1;
            inst := *vm.program.insts[at];
            if inst.kind == {
                case .MATCH; #through;
                case .RANGES;
                    keep := thread_slots(list, at);
                    for slots keep[it_index] = it;

                    break;
                case .SPLIT;
                    array_add(stack, .{ pc = inst.alt, restore = -1 });
                    at = inst.next;
                case .SAVE;
                    if inst.alt < cast(u32) vm.slot_count {
                        array_add(stack, .{ restore = cast(int) inst.alt, value = slots[inst.alt] });
                        slots[inst.alt] = pos;
                    }

                    at = inst.next;
                case .LOOK;
                    if !look_holds(inst.look, vm.text, pos) break;

                    at = inst.next;
                case .FAIL;
                    break;
            }
        }
    }
}
