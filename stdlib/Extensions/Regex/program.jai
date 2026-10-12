// Compiles a parsed pattern into a Program: a byte-level automaton with captures, the form every
// matcher runs. Code point classes become UTF-8 byte paths here, so the matchers only ever see
// bytes. The same tree can also be compiled reversed, for the matcher that finds where a match
// starts by walking back from its end.

Inst_Kind :: enum u8 {
    MATCH;
    RANGES; // consume one byte that falls in one of transitions[next .. next + alt]
    SPLIT; // try `next`, then `alt`
    SAVE; // record the position in slot `alt`, go on to `next`
    LOOK; // go on to `next` if `look` holds here
    FAIL;
}

Inst :: struct {
    kind: Inst_Kind;
    look: Look;
    next: u32;
    alt: u32;
}

Transition :: struct {
    lo: u8;
    hi: u8;
    next: u32;
}

Program :: struct {
    insts: [] Inst;
    transitions: [] Transition;
    start: u32; // anchored: tries to match right here
    start_unanchored: u32; // lets the match begin at any later byte, at lower priority
    slot_count: int; // 2 per group, counting the whole match as group 0
    names: [] string; // names[i] is group i's name, "" if unnamed
    anchored: bool; // every match starts at the start of the text (\A)
    has_looks: bool;
    has_unicode_word: bool; // \b and friends in Unicode mode, which byte automata can't see
    byte_class: [256] u8; // bytes that the program never tells apart share a class
    class_count: int;
    class_byte: [256] u8; // one byte of each class
}

MAX_INSTS :: 1_000_000;

// Compiles `root`. `reverse` builds the program that reads the text backwards (no captures
// and no unanchored start: it is only run back from the end of a match).
compile_program :: (parsed: *Parsed, reverse: bool) -> Program, ok: bool, error: string {
    c: Compiler;
    c.reverse = reverse;
    init(*c.shared, 64);
    match := emit(*c, .{ kind = .MATCH });
    entry: u32;

    if reverse {
        entry = compile_node(*c, parsed.root, match);
    } else {
        end := emit(*c, .{ kind = .SAVE, next = match, alt = 1 });
        body := compile_node(*c, parsed.root, end);
        entry = emit(*c, .{ kind = .SAVE, next = body, alt = 0 });
    }

    for c.shared free(it_index);

    deinit(*c.shared);
    out: Program;
    if c.too_big return out, false, "pattern compiles to too large a program";

    out.start = entry;

    if reverse {
        out.start_unanchored = entry;
    } else {
        // loop: SPLIT(entry, any) ; any: [00-FF] -> loop. The lazy loop keeps later starts
        // below every thread that started earlier.
        loop := emit(*c, .{ kind = .SPLIT, next = entry });
        first := cast(u32) c.transitions.count;

        array_add(*c.transitions, .{ 0, 255, loop });
        any := emit(*c, .{ kind = .RANGES, next = first, alt = 1 });

        c.insts[loop].alt = any;
        out.start_unanchored = loop;
    }

    out.insts = c.insts;
    out.transitions = c.transitions;
    out.slot_count = 2 * (parsed.groups + 1);
    out.names = parsed.names;
    out.anchored = starts_with_text_start(parsed.root);

    for c.insts if it.kind == .LOOK {
        out.has_looks = true;
        if it.look >= .WORD && it.look <= .WORD_END_HALF out.has_unicode_word = true;
    }

    compute_byte_classes(*out);

    return out, true, "";
}

// Every match begins at the start of the text.
starts_with_text_start :: (node: *Node) -> bool {
    if node.kind == {
        case .LOOK;
            return node.look == .START_TEXT;
        case .CAPTURE;
            return starts_with_text_start(node.children[0]);
        case .CONCAT;
            for node.children {
                if starts_with_text_start(it) return true;
                if !matches_only_empty(it) return false;
            }

            return false;
        case .ALTERNATE;
            for node.children if !starts_with_text_start(it) return false;

            return true;
        case .REPEAT;
            return node.min > 0 && starts_with_text_start(node.children[0]);
    }

    return false;
}

// Can only ever match the empty string at a position (a look or nothing), so it can be skipped
// over when looking for what a concatenation starts with.
matches_only_empty :: (node: *Node) -> bool {
    if node.kind == .EMPTY return true;
    if node.kind == .CAPTURE return matches_only_empty(node.children[0]);
    return false;
}

can_match_empty :: (node: *Node) -> bool {
    if node.kind == {
        case .EMPTY; #through;
        case .LOOK; return true;
        case .LITERAL; return node.literal.count == 0;
        case .CLASS; return false;
        case .REPEAT; return node.min == 0 || can_match_empty(node.children[0]);
        case .CAPTURE; return can_match_empty(node.children[0]);
        case .CONCAT;
            for node.children if !can_match_empty(it) return false;

            return true;
        case .ALTERNATE;
            for node.children if can_match_empty(it) return true;

            return false;
    }

    return false;
}

#scope_file

Compiler :: struct {
    reverse: bool;
    insts: [..] Inst;
    transitions: [..] Transition;
    shared: Table(string, u32); // byte trie nodes already emitted, by their transitions
    too_big: bool;
}

emit :: (c: *Compiler, inst: Inst) -> u32 {
    if c.insts.count >= MAX_INSTS {
        c.too_big = true;
        return 0;
    }

    array_add(*c.insts, inst);

    return cast(u32) (c.insts.count - 1);
}

emit_ranges :: (c: *Compiler, ranges: [] Transition) -> u32 {
    first := cast(u32) c.transitions.count;
    for ranges array_add(*c.transitions, it);
    return emit(c, .{ kind = .RANGES, next = first, alt = cast(u32) ranges.count });
}

// Compiles `node` so that it continues to `next`; returns where it starts. Building from the
// end means every continuation already exists when something jumps to it.
compile_node :: (c: *Compiler, node: *Node, next: u32) -> u32 {
    if c.too_big return 0;

    if node.kind == {
        case .EMPTY;
            return next;
        case .LITERAL;
            at := next;
            text := node.literal;

            if c.reverse {
                for text at = emit_ranges(c, .[.{ it, it, at }]);
            } else {
                for < text at = emit_ranges(c, .[.{ it, it, at }]);
            }

            return at;
        case .CLASS;
            return compile_class(c, node, next);
        case .LOOK;
            return emit(c, .{ kind = .LOOK, look = node.look, next = next });
        case .CAPTURE;
            if c.reverse return compile_node(c, node.children[0], next);

            close := emit(c, .{ kind = .SAVE, next = next, alt = cast(u32) (2 * node.index + 1) });
            body := compile_node(c, node.children[0], close);

            return emit(c, .{ kind = .SAVE, next = body, alt = cast(u32) (2 * node.index) });
        case .CONCAT;
            at := next;

            if c.reverse {
                for node.children at = compile_node(c, it, at);
            } else {
                for < node.children at = compile_node(c, it, at);
            }

            return at;
        case .ALTERNATE;
            entries: [..] u32;

            defer array_free(entries);
            for node.children array_add(*entries, compile_node(c, it, next));

            at := entries[entries.count - 1];
            for < 0..entries.count - 2 at = emit(c, .{ kind = .SPLIT, next = entries[it], alt = at });

            return at;
        case .REPEAT;
            return compile_repeat(c, node, next);
    }

    return next;
}

split :: (c: *Compiler, greedy: bool, body: u32, skip: u32) -> u32 {
    if greedy return emit(c, .{ kind = .SPLIT, next = body, alt = skip });
    return emit(c, .{ kind = .SPLIT, next = skip, alt = body });
}

compile_repeat :: (c: *Compiler, node: *Node, next: u32) -> u32 {
    child := node.children[0];
    at := next;
    mandatory := node.min;

    if node.max < 0 && node.min == 0 && can_match_empty(child) {
        // x* where x can match empty is (x+)?, so that a single empty pass through x still
        // sets x's groups, as Perl-style engines do.
        loop := emit(c, .{ kind = .SPLIT });
        body := compile_node(c, child, loop);

        if c.too_big return 0;
        if node.greedy c.insts[loop] = .{ kind = .SPLIT, next = body, alt = next };
        else c.insts[loop] = .{ kind = .SPLIT, next = next, alt = body };

        return split(c, node.greedy, body, next);
    }

    if node.max < 0 {
        // The last mandatory copy (or an optional one, for min 0) loops back on itself.
        loop := emit(c, .{ kind = .SPLIT });
        body := compile_node(c, child, loop);

        if c.too_big return 0;
        if node.greedy c.insts[loop] = .{ kind = .SPLIT, next = body, alt = next };
        else c.insts[loop] = .{ kind = .SPLIT, next = next, alt = body };

        if mandatory > 0 {
            at = body;
            mandatory -= 1;
        } else {
            at = loop;
        }
    } else {
        for 1..node.max - node.min {
            body := compile_node(c, child, at);
            at = split(c, node.greedy, body, next);
            if c.too_big return 0;
        }
    }

    for 1..mandatory {
        at = compile_node(c, child, at);
        if c.too_big return 0;
    }

    return at;
}

// A class of bytes is one RANGES instruction. A class of code points is a trie of UTF-8 byte
// paths; trie nodes with the same transitions are emitted once, which keeps big classes like
// \w small and lets them share their continuation-byte tails.
compile_class :: (c: *Compiler, node: *Node, next: u32) -> u32 {
    if node.ranges.count == 0 return emit(c, .{ kind = .FAIL });

    if node.bytes {
        list: [..] Transition;
        defer array_free(list);
        for node.ranges array_add(*list, .{ cast(u8) it.lo, cast(u8) it.hi, next });
        return emit_ranges(c, list);
    }

    root := New(Trie_Node);

    defer free_trie(root);

    for node.ranges {
        for_utf8_sequences(it.lo, it.hi, root, c.reverse);
    }

    return emit_trie(c, root, next);
}

Byte_Range :: struct {
    lo: u8;
    hi: u8;
}

Trie_Node :: struct {
    edges: [..] Trie_Edge;
}

Trie_Edge :: struct {
    lo: u8;
    hi: u8;
    child: *Trie_Node; // null on the last byte of a path
}

free_trie :: (node: *Trie_Node) {
    for node.edges if it.child free_trie(it.child);
    array_free(node.edges);
    free(node);
}

trie_insert :: (root: *Trie_Node, path: [] Byte_Range) {
    node := root;

    for path {
        last := it_index == path.count - 1;
        found: *Trie_Edge;

        for *edge: node.edges {
            if edge.lo == it.lo && edge.hi == it.hi && (edge.child != null) == !last {
                found = edge;
                break;
            }
        }

        if !found {
            array_add(*node.edges, .{ lo = it.lo, hi = it.hi, child = ifx last then null else New(Trie_Node) });
            found = *node.edges[node.edges.count - 1];
        }

        node = found.child;
    }
}

emit_trie :: (c: *Compiler, node: *Trie_Node, next: u32) -> u32 {
    list: [..] Transition;

    defer array_free(list);

    for node.edges {
        target := ifx it.child then emit_trie(c, it.child, next) else next;
        array_add(*list, .{ it.lo, it.hi, target });
    }

    // Same transitions, same node.
    builder: String_Builder;
    for list print_to_builder(*builder, "%-%>%,", it.lo, it.hi, it.next);

    key := builder_to_string(*builder);
    found, existing := table_find(*c.shared, key);

    if found {
        free(key);
        return existing;
    }

    at := emit_ranges(c, list);

    table_add(*c.shared, key, at);

    return at;
}

// Splits [lo, hi] into runs whose UTF-8 encodings are each a fixed sequence of byte ranges, and
// adds each run's path to the trie. Surrogates have no UTF-8 form and are skipped.
for_utf8_sequences :: (lo: u32, hi: u32, root: *Trie_Node, reverse: bool) {
    if lo > hi return;

    if lo <= 0xDFFF && hi >= 0xD800 {
        if lo < 0xD800 for_utf8_sequences(lo, 0xD7FF, root, reverse);
        if hi > 0xDFFF for_utf8_sequences(0xE000, hi, root, reverse);
        return;
    }

    // Split where the encoded length changes.
    LIMITS :: u32.[0x7F, 0x7FF, 0xFFFF];
    for LIMITS {
        if lo <= it && hi > it {
            for_utf8_sequences(lo, it, root, reverse);
            for_utf8_sequences(it + 1, hi, root, reverse);
            return;
        }
    }

    if hi < 0x80 {
        path := Byte_Range.[.{ cast(u8) lo, cast(u8) hi }];
        trie_insert(root, path);
        return;
    }

    count := ifx hi < 0x800 then 2 else ifx hi < 0x10000 then 3 else 4;
    // Where lo and hi differ above the low 6*i bits, the low bits must cover everything, or
    // the run is split so that they do.
    for i: 1..count - 1 {
        mask := (cast(u32) 1 << cast(u32) (6 * i)) - 1;
        if (lo & ~mask) != (hi & ~mask) {
            if (lo & mask) != 0 {
                for_utf8_sequences(lo, lo | mask, root, reverse);
                for_utf8_sequences((lo | mask) + 1, hi, root, reverse);
                return;
            }

            if (hi & mask) != mask {
                for_utf8_sequences(lo, (hi & ~mask) - 1, root, reverse);
                for_utf8_sequences(hi & ~mask, hi, root, reverse);
                return;
            }
        }
    }

    a, b: [4] u8;

    utf8_encode(lo, *a);
    utf8_encode(hi, *b);
    path: [4] Byte_Range;

    for 0..count - 1 {
        index := ifx reverse then count - 1 - it else it;
        path[it] = .{ a[index], b[index] };
    }

    view: [] Byte_Range;
    view.data = path.data;
    view.count = count;
    trie_insert(root, view);
}

// Byte classes: two bytes share a class when no transition and no look can tell them apart.
compute_byte_classes :: (p: *Program) {
    boundary: [257] bool;

    mark :: (lo: int, hi: int) #expand {
        boundary[lo] = true;
        boundary[hi + 1] = true;
    }

    for p.transitions mark(it.lo, it.hi);

    if p.has_looks {
        mark(#char "\n", #char "\n");
        mark(#char "\r", #char "\r");
        mark(#char "0", #char "9");
        mark(#char "A", #char "Z");
        mark(#char "_", #char "_");
        mark(#char "a", #char "z");
        mark(0x80, 0xFF);
    }

    class := 0;

    for b: 0..255 {
        if b > 0 && boundary[b] class += 1;
        p.byte_class[b] = cast(u8) class;
    }

    p.class_count = class + 1;
    for < b: 0..255 p.class_byte[p.byte_class[b]] = cast(u8) b;
}
