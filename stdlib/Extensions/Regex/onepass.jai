// The one-pass capture engine. A pattern is one-pass when, matching anchored at a known start,
// every byte leaves at most one way forward through the program. For those patterns the group
// offsets can be read off in a single scan of the match, with no backtracking and no thread
// lists: a table says, for each (state, byte class), which state comes next and which capture
// slots to set on the way.
//
// A state is the entry point of a stretch of the program that waits for a byte (the program start,
// or the target of a byte transition). Following SPLIT, SAVE and LOOK from it, in priority order
// and visiting each instruction once as the Pike VM does, gives the byte-consuming instructions
// ("leaves") that can run next. If two leaves accept the same byte class, the pattern is not
// one-pass. A MATCH leaf cuts off every leaf of lower priority, which is how leftmost-first
// preference is kept.
//
// Looks stay as conditions on the edge that crosses them and are checked against the text when
// the edge is taken. A look makes the choice depend on the text, not just on the byte, so where
// that would change which leaf wins (see `explore`), the pattern is refused.

OnePass :: struct {
    ok: bool; // false: not one-pass, or too big; nothing below is set
    stride: int; // byte classes
    slot_count: int;
    start: int; // premultiplied by stride
    byte_class: [256] u8;
    trans: [] Edge; // [state * stride + class]
    finals: [] Edge; // [state]: `next` is 1 if the match can end in this state
    updates: [] u32; // lists of slots to set: a count, then the slots. Entry 0 is the empty list.
}

Edge :: struct {
    next: u32; // premultiplied by stride; NO_EDGE if the byte is not allowed
    updates: u32; // index into OnePass.updates
    need: u32; // bit set of the Looks that must hold before this edge is taken
}

NO_EDGE :: 0xFFFF_FFFF;

// Run-time engines may be bigger than ones frozen into a program.
ONEPASS_STATE_LIMIT :: 1000;
ONEPASS_STATIC_STATE_LIMIT :: 256;

// Builds the table for `program` (the forward one). `ok` is false in the result if the pattern
// is not one-pass or needs more than `state_limit` states.
onepass_build :: (program: *Program, state_limit: int) -> OnePass {
    out: OnePass;
    if program.insts.count > 100_000 return out;

    b: Builder;
    b.program = program;
    b.stride = program.class_count;
    b.limit = state_limit;
    b.state_of = NewArray(program.insts.count, u32);
    for *b.state_of it.* = NO_EDGE;

    b.seen = NewArray(program.insts.count, u32);
    b.seen_need = NewArray(program.insts.count, u32);
    array_add(*b.updates, 0);

    defer {
        array_free(b.entries);
        array_free(b.trans);
        array_free(b.finals);
        array_free(b.updates);
        array_free(b.stack);
        array_free(b.path);
        array_free(b.leaves);
        array_free(b.state_of);
        array_free(b.seen);
        array_free(b.seen_need);
    }

    start := state_for(*b, program.start);
    i := 0;

    while i < b.entries.count {
        if !fill_state(*b, i) return out;
        i += 1;
    }

    if b.failed return out;

    out.ok = true;
    out.stride = b.stride;
    out.slot_count = program.slot_count;
    out.start = cast(int) start;
    out.byte_class = program.byte_class;
    out.trans = NewArray(b.trans.count, Edge);
    for b.trans out.trans[it_index] = it;

    out.finals = NewArray(b.finals.count, Edge);
    for b.finals out.finals[it_index] = it;

    out.updates = NewArray(b.updates.count, u32);
    for b.updates out.updates[it_index] = it;

    return out;
}

onepass_free :: (t: *OnePass) {
    if !t.ok return;

    array_free(t.trans);
    array_free(t.finals);
    array_free(t.updates);
    t.* = .{};
} @Releases

// Fills `slots` (-1 where a group did not take part) with the offsets of the match
// text[s .. e] that starts at `s`, which must be the match the pattern prefers there. False if
// the table finds no path that ends exactly at `e`.
onepass_search :: (t: *OnePass, text: string, s: int, e: int, slots: [] int) -> bool {
    for *slots it.* = -1;

    state := t.start;
    pos := s;

    while pos < e {
        edge := t.trans[state + cast(int) t.byte_class[text[pos]]];
        if edge.next == NO_EDGE return false;
        if edge.need != 0 && !looks_hold(edge.need, text, pos) return false;
        if edge.updates != 0 apply_updates(t, edge.updates, pos, slots);

        state = cast(int) edge.next;
        pos += 1;
    }

    final := t.finals[state / t.stride];
    if final.next == 0 return false;
    if final.need != 0 && !looks_hold(final.need, text, pos) return false;
    if final.updates != 0 apply_updates(t, final.updates, pos, slots);

    return true;
}

#scope_file

Builder :: struct {
    program: *Program;
    stride: int;
    limit: int;
    failed: bool;
    state_of: [] u32; // instruction -> its state, NO_EDGE if it has none (yet)
    entries: [..] u32; // state -> the instruction it starts at
    trans: [..] Edge;
    finals: [..] Edge;
    updates: [..] u32;

    // For one closure:
    seen: [] u32; // the generation that last reached each instruction
    seen_need: [] u32; // and the looks it crossed on the way
    generation: u32;
    stack: [..] Frame;
    path: [..] u32; // slots saved on the way to here
    leaves: [..] Leaf;
}

Frame :: struct {
    pc: u32;
    path_length: int;
    need: u32;
}

Leaf :: struct {
    pc: u32;
    updates: u32;
    need: u32;
}

// The state that starts at `pc`, made if it is new.
state_for :: (b: *Builder, pc: u32) -> u32 {
    if b.state_of[pc] != NO_EDGE return b.state_of[pc];

    if b.entries.count >= b.limit {
        b.failed = true;
        return 0;
    }

    at := cast(u32) (b.entries.count * b.stride);

    b.state_of[pc] = at;
    array_add(*b.entries, pc);

    return at;
}

// Computes the edges out of state `index`. False if the state breaks the one-pass rule.
fill_state :: (b: *Builder, index: int) -> bool {
    program := b.program;
    row := index * b.stride;

    for 1..b.stride array_add(*b.trans, .{ NO_EDGE, 0, 0 });

    array_add(*b.finals, .{ 0, 0, 0 });
    if !explore(b, b.entries[index]) return false;

    for leaf: b.leaves {
        inst := program.insts[leaf.pc];
        if inst.kind == .MATCH {
            b.finals[index] = .{ 1, leaf.updates, leaf.need };
            continue;
        }

        for t: 0..inst.alt - 1 {
            tr := program.transitions[inst.next + t];
            target: u32 = NO_EDGE;

            for c: 0..b.stride - 1 {
                byte := program.class_byte[c];
                if byte < tr.lo || byte > tr.hi continue;
                if b.trans[row + c].next != NO_EDGE return false; // two ways to take this byte

                if target == NO_EDGE {
                    target = state_for(b, tr.next);
                    if b.failed return false;
                }

                b.trans[row + c] = .{ target, leaf.updates, leaf.need };
            }
        }
    }

    return true;
}

// Finds the leaves reachable from `entry` without consuming a byte, in priority order, with the
// slots to set and the looks to pass on the way to each. False if the choice among them could
// depend on the text.
explore :: (b: *Builder, entry: u32) -> bool {
    program := b.program;

    b.generation += 1;
    b.leaves.count = 0;
    b.path.count = 0;
    b.stack.count = 0;
    array_add(*b.stack, .{ pc = entry, path_length = 0, need = 0 });
    guarded_match := false; // a MATCH behind a look has been seen: it may not apply

    while b.stack.count {
        frame := pop(*b.stack);

        b.path.count = frame.path_length;
        need := frame.need;
        pc := frame.pc;

        while true {
            if b.seen[pc] == b.generation {
                // The Pike VM keeps the first visit. That is only the answer here if the first
                // visit could not have been cut short by a look.
                if b.seen_need[pc] != 0 return false;
                break;
            }

            b.seen[pc] = b.generation;
            b.seen_need[pc] = need;
            inst := *program.insts[pc];
            if inst.kind == {
                case .MATCH;
                    if guarded_match return false;

                    array_add(*b.leaves, .{ pc, intern_updates(b), need });

                    if need == 0 {
                        // Nothing of lower priority can win over a match that always applies.
                        b.stack.count = 0;
                    } else {
                        guarded_match = true;
                    }

                    break;
                case .RANGES;
                    if guarded_match return false;

                    array_add(*b.leaves, .{ pc, intern_updates(b), need });
                    break;
                case .SPLIT;
                    array_add(*b.stack, .{ pc = inst.alt, path_length = b.path.count, need = need });
                    pc = inst.next;
                case .SAVE;
                    array_add(*b.path, inst.alt);
                    pc = inst.next;
                case .LOOK;
                    need |= cast(u32) 1 << cast(u32) inst.look;
                    pc = inst.next;
                case .FAIL;
                    break;
            }
        }
    }

    return true;
}

// Stores the current path's slots as a sorted list without repeats; returns its index.
intern_updates :: (b: *Builder) -> u32 {
    if b.path.count == 0 return 0;

    list: [..] u32;

    defer array_free(list);

    for slot: b.path {
        at := 0;
        while at < list.count && list[at] < slot at += 1;
        if at < list.count && list[at] == slot continue;

        array_add(*list, slot);
        k := list.count - 1;
        while k > at {
            list[k] = list[k - 1];
            k -= 1;
        }

        list[at] = slot;
    }

    index := cast(u32) b.updates.count;

    array_add(*b.updates, cast(u32) list.count);
    for list array_add(*b.updates, it);

    return index;
}

apply_updates :: inline (t: *OnePass, index: u32, pos: int, slots: [] int) {
    count := t.updates[index];
    for 1..count {
        slot := t.updates[index + it];
        if slot < cast(u32) slots.count slots[slot] = pos;
    }
}

// Whether every look in `need` holds at `pos`.
looks_hold :: (need: u32, text: string, pos: int) -> bool {
    for 0..cast(int) Look.WORD_END_HALF_ASCII {
        if need & (cast(u32) 1 << cast(u32) it) {
            if !look_holds(cast(Look) it, text, pos) return false;
        }
    }

    return true;
}
