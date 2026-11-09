Segmentation :: enum u8 {
    GRAPHEME;
    WORD;
    SENTENCE;
}

// Borrows its text and allocates nothing. Check `error` after iteration, including empty input.
Segment_Iterator :: struct {
    text: string;
    kind: Segmentation;
    offset: int;
    error: Unicode_Text_Error;
    lookahead := -1;
    lookahead_class: S;
}

// Creates a byte-offset iterator. Words include punctuation and spaces; no dictionary tailoring.
// Example: `iter := segments("hello world", .WORD);` then call `next_segment(*iter)`.
segments :: (text: string, kind: Segmentation, options: Unicode_Text_Options = .{}) -> Segment_Iterator {
    result := Segment_Iterator.{ text = text, kind = kind };
    result.error = arguments_error(text, .NFC, options);
    if cast(int) kind > 2 result.error = .{ .INVALID_ARGUMENT, "invalid segmentation kind", 0 };
    return result;
}

// Returns [start, end) and true, or false at EOF/error. Each successful call advances.
// Example: `start, end, more := next_segment(*iter);` slices the next segment by byte offsets.
next_segment :: (iter: *Segment_Iterator) -> start: int, end: int, more: bool {
    if iter.error.kind != .NONE || iter.offset >= iter.text.count return iter.offset, iter.offset, false;

    start = iter.offset;
    state: Break_State;

    while iter.offset < iter.text.count {
        cp, size, valid := decode(iter.text, iter.offset);
        if !valid {
            iter.error = .{ .INVALID_UTF8, "invalid UTF-8", iter.offset };
            return start, iter.offset, false;
        }

        p := property(cp);

        if iter.offset > start {
            joins := false;
            if iter.kind == .GRAPHEME joins = grapheme_join(*state, p);
            else if iter.kind == .WORD joins = word_join(iter, *state, p, size);
            else joins = sentence_join(iter, *state, p);
            if iter.error.kind != .NONE return start, iter.offset, false;
            if !joins return start, iter.offset, true;
        }

        advance_state(*state, p, iter.kind);
        iter.offset += size;
    }

    return start, iter.offset, true;
}

// Finds the cluster end from a known boundary; decodes only through that cluster and its lookahead.
// Example: `end, error := next_grapheme("éx", 0);` gives 3.
next_grapheme :: (text: string, index: int, options: Unicode_Text_Options = .{}) -> int, Unicode_Text_Error {
    if index < 0 || index > text.count return index, .{ .INVALID_ARGUMENT, "invalid byte offset", index };

    iter := segments(text, .GRAPHEME, options);
    iter.offset = index;
    _, end, _ := next_segment(*iter);

    return end, iter.error;
}

// Finds the cluster start before `index`, backing up to a guaranteed boundary for context.
// Example: `start, error := previous_grapheme("éx", 3);` gives 0.
previous_grapheme :: (text: string, index: int, options: Unicode_Text_Options = .{}) -> int, Unicode_Text_Error {
    if index < 0 || index > text.count return index, .{ .INVALID_ARGUMENT, "invalid byte offset", index };

    iter := segments(text, .GRAPHEME, options);
    if iter.error.kind != .NONE return index, iter.error;

    safe := index;
    right: u32;

    while safe > 0 {
        left := safe - 1;
        while left > 0 && (text[left] & 0xC0) == 0x80 && safe - left < 4 left -= 1;

        cp, size, valid := decode(text, left);
        if !valid || left + size != safe return left, .{ .INVALID_UTF8, "invalid UTF-8", left };

        p := property(cp);

        if safe < index {
            // Assume every context-sensitive joining rule can match; only a certain break is safe.
            maximal := Break_State.{ g = gclass(p), regional = 1, consonant = true, linker = true, zwj_pictographic = gclass(p) == .ZWJ };
            if !grapheme_join(*maximal, right) break;
        }

        safe = left;
        right = p;
    }

    iter.offset = safe;
    while iter.offset < index {
        start, end, more := next_segment(*iter);
        if !more return start, iter.error;
        if end >= index return start, .{};
    }

    return 0, iter.error;
}

// Counts extended grapheme clusters without allocation.
// Example: `count, error := grapheme_count("éx");` gives 2.
grapheme_count :: (text: string, options: Unicode_Text_Options = .{}) -> int, Unicode_Text_Error {
    iter := segments(text, .GRAPHEME, options);
    count := 0;

    while true {
        _, _, more := next_segment(*iter);
        if !more break;
        count += 1;
    }

    return count, iter.error;
}

#scope_module

Break_State :: struct {
    g: G;
    w: W;
    w_before: W;
    immediate_w: W;
    s: S;
    s_before: S;
    immediate_s: S;
    regional: int;
    pictographic: bool;
    zwj_pictographic: bool;
    consonant: bool;
    linker: bool;
    terminal: S;
    phase: int; // 1: terminal Close*, 2: terminal Close* Sp*.
}

gclass :: inline (p: u32) -> G {
    return cast(G) ((p >> 8) & 15);
}

wclass :: inline (p: u32) -> W {
    return cast(W) ((p >> 12) & 31);
}

sclass :: inline (p: u32) -> S {
    return cast(S) ((p >> 17) & 15);
}

pictographic :: inline (p: u32) -> bool {
    return (p & (1 << 23)) != 0;
}

gcontrol :: (g: G) -> bool {
    return g == .CR || g == .LF || g == .Control;
}

wnewline :: (w: W) -> bool {
    return w == .CR || w == .LF || w == .Newline;
}

wignore :: (w: W) -> bool {
    return w == .Extend || w == .Format || w == .ZWJ;
}

letter :: (w: W) -> bool {
    return w == .ALetter || w == .Hebrew_Letter;
}

midletter :: (w: W) -> bool {
    return w == .MidLetter || w == .MidNumLet || w == .Single_Quote;
}

midnumber :: (w: W) -> bool {
    return w == .MidNum || w == .MidNumLet || w == .Single_Quote;
}

wordish :: (w: W) -> bool {
    return letter(w) || w == .Numeric || w == .Katakana || w == .ExtendNumLet;
}

spara :: (s: S) -> bool {
    return s == .CR || s == .LF || s == .Sep;
}

sterm :: (s: S) -> bool {
    return s == .ATerm || s == .STerm;
}

signore :: (s: S) -> bool {
    return s == .Extend || s == .Format;
}

grapheme_join :: (s: *Break_State, p: u32) -> bool {
    a := s.g;
    b := gclass(p);

    if a == .CR && b == .LF return true;
    if gcontrol(a) || gcontrol(b) return false;
    if a == .L && (b == .L || b == .V || b == .LV || b == .LVT) return true;
    if (a == .LV || a == .V) && (b == .V || b == .T) return true;
    if (a == .LVT || a == .T) && b == .T return true;
    if b == .Extend || b == .ZWJ || b == .SpacingMark || a == .Prepend return true;
    if ((p >> 21) & 3) == 1 && s.consonant && s.linker return true;
    if pictographic(p) && s.zwj_pictographic return true;

    return a == .Regional_Indicator && b == .Regional_Indicator && s.regional % 2 == 1;
}

word_lookahead :: (iter: *Segment_Iterator, at: int) -> W {
    pos := at;
    while pos < iter.text.count {
        cp, size, valid := decode(iter.text, pos);
        if !valid {
            iter.error = .{ .INVALID_UTF8, "invalid UTF-8", pos };
            return .Other;
        }

        w := wclass(property(cp));
        if !wignore(w) return w;

        pos += size;
    }

    return .Other;
}

word_join :: (iter: *Segment_Iterator, s: *Break_State, p: u32, size: int) -> bool {
    b := wclass(p);
    raw := s.immediate_w;

    if raw == .CR && b == .LF return true;
    if wnewline(raw) || wnewline(b) return false;
    if raw == .ZWJ && pictographic(p) return true;
    if raw == .WSegSpace && b == .WSegSpace return true;
    if wignore(b) return true;

    a := s.w;
    before := s.w_before;

    if letter(a) && letter(b) return true;
    if letter(a) && midletter(b) && letter(word_lookahead(iter, iter.offset + size)) return true;
    if letter(before) && midletter(a) && letter(b) return true;
    if a == .Hebrew_Letter && b == .Single_Quote return true;
    if a == .Hebrew_Letter && b == .Double_Quote && word_lookahead(iter, iter.offset + size) == .Hebrew_Letter return true;
    if before == .Hebrew_Letter && a == .Double_Quote && b == .Hebrew_Letter return true;
    if a == .Numeric && b == .Numeric return true;
    if (letter(a) && b == .Numeric) || (a == .Numeric && letter(b)) return true;
    if before == .Numeric && midnumber(a) && b == .Numeric return true;
    if a == .Numeric && midnumber(b) && word_lookahead(iter, iter.offset + size) == .Numeric return true;
    if a == .Katakana && b == .Katakana return true;
    if wordish(a) && b == .ExtendNumLet return true;
    if a == .ExtendNumLet && wordish(b) return true;

    return a == .Regional_Indicator && b == .Regional_Indicator && s.regional % 2 == 1;
}

sentence_lower_ahead :: (iter: *Segment_Iterator) -> bool {
    // Cache the next SB8 stopping class so a long punctuation/space run is scanned only once.
    if iter.lookahead < iter.offset {
        pos := iter.offset;

        iter.lookahead_class = .Other;
        while pos < iter.text.count {
            cp, size, valid := decode(iter.text, pos);
            if !valid {
                iter.error = .{ .INVALID_UTF8, "invalid UTF-8", pos };
                return false;
            }

            c := sclass(property(cp));
            if c == .OLetter || c == .Upper || c == .Lower || spara(c) || sterm(c) {
                iter.lookahead_class = c;
                break;
            }

            pos += size;
        }

        iter.lookahead = pos;
    }

    return iter.lookahead_class == .Lower;
}

sentence_join :: (iter: *Segment_Iterator, s: *Break_State, p: u32) -> bool {
    b := sclass(p);
    if s.immediate_s == .CR && b == .LF return true;
    if spara(s.immediate_s) return false;
    if signore(b) return true;
    if s.s == .ATerm && b == .Numeric return true;
    if (s.s_before == .Upper || s.s_before == .Lower) && s.s == .ATerm && b == .Upper return true;
    if s.phase == 0 return true;
    if s.terminal == .ATerm && sentence_lower_ahead(iter) return true;
    if b == .SContinue || sterm(b) return true;
    if s.phase == 1 && (b == .Close || b == .Sp || spara(b)) return true;
    if s.phase == 2 && (b == .Sp || spara(b)) return true;

    return false;
}

advance_state :: (s: *Break_State, p: u32, kind: Segmentation) {
    if kind == .GRAPHEME {
        g := gclass(p);
        if g == .Regional_Indicator s.regional += 1;
        else s.regional = 0;

        s.zwj_pictographic = g == .ZWJ && s.pictographic;
        if g != .Extend s.pictographic = pictographic(p);

        incb := (p >> 21) & 3;
        if incb == 1 {
            s.consonant = true;
            s.linker = false;
        } else if incb == 3 {
            s.linker = s.consonant;
        } else if incb != 2 {
            s.consonant = false;
            s.linker = false;
        }

        s.g = g;
    } else if kind == .WORD {
        w := wclass(p);

        s.immediate_w = w;

        if !wignore(w) {
            s.w_before = s.w;
            s.w = w;
            if w == .Regional_Indicator s.regional += 1;
            else s.regional = 0;
        }
    } else {
        c := sclass(p);

        s.immediate_s = c;
        if signore(c) return;

        s.s_before = s.s;
        s.s = c;

        if sterm(c) {
            s.terminal = c;
            s.phase = 1;
        } else if c == .Close && s.phase == 1 {} else if c == .Sp && s.phase != 0 {
            s.phase = 2;
        } else s.phase = 0;
    }
}
