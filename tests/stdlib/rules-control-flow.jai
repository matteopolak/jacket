// Checks the claims of docs/language/control-flow.md, one group per rule.
// rules: flow.1 flow.2 flow.3 flow.4 flow.5 flow.6 flow.7 flow.8 flow.9 flow.10 flow.11 flow.12 flow.13 flow.14
// rules: flow.16 flow.17 flow.18 flow.19 flow.20 flow.21 flow.22 flow.23 flow.24 flow.25
#import "Basic";

Hue :: enum {
    RED;
    GREEN;
    BLUE;
    CRIMSON :: 0;
}
Bits :: enum_flags {
    A;
    B;
    C;
}

trace: [..] int;

note :: (v: int) -> int {
    array_add(*trace, v);
    return v;
}

Countdown :: struct {
    from: int;
}

for_expansion :: (c: Countdown, body: Code, flags: For_Flags) #expand {
    n := c.from;
    while n > 0 {
        defer n -= 1;
        `it := n;
        `it_index := c.from - n;
        #insert (break = break, continue = continue) body;
    }
}

Halt :: struct {
    limit: int;
}

stopped_by_macro := false;

for_expansion :: (h: Halt, body: Code, flags: For_Flags) #expand {
    i := 0;
    while i < h.limit {
        `it := i;
        `it_index := i;
        i += 1;
        #insert (break = {
            stopped_by_macro = true;
            break;
        }) body;
    }
}

defer_log: [..] string;

defer_return :: () {
    defer array_add(*defer_log, "outer");
    {
        defer array_add(*defer_log, "inner");
        return;
    }
}

main :: () {
    // flow.1: `for <` visits the range from its end down to its start.
    seen: [..] int;
    for < 1..3 array_add(*seen, it);
    assert(seen.count == 3 && seen[0] == 3 && seen[1] == 2 && seen[2] == 1, "reverse range is 3 2 1");

    // flow.2: `continue outer` continues the named outer loop.
    total := 0;
    for outer: 1..4 {
        for < inner: 1..3 {
            if inner == 2 continue outer;
            total += outer;
        }
    }
    assert(total == 10, "named continue gives 10");

    // flow.3: a range whose end is below its start runs zero times.
    runs := 0;
    for 5..3 runs += 1;
    assert(runs == 0, "descending range is empty");

    // flow.4: `for v, i:` binds the value then the index.
    arr := int.[10, 20, 30];
    sum_vi := 0;
    for v, i: arr sum_vi += v * (i + 1);
    assert(sum_vi == 10 + 40 + 90, "value then index");

    // flow.5: pointer iteration writes through to the array.
    nums: [3] int = .[1, 2, 3];
    for *p: nums p.* += 1;
    assert(nums[0] == 2 && nums[1] == 3 && nums[2] == 4, "pointer iteration writes through");

    // flow.6, flow.24: `while x := cond` names the loop and binds the condition's value.
    j := 0;
    while x := j < 3 {
        assert(x, "binding holds the condition");
        j += 1;
    }
    assert(j == 3, "named while ran 3 times");
    k := 0;
    while going := k < 10 {
        k += 1;
        if k == 4 break going;
    }
    assert(k == 4, "break by while name");

    // flow.25: `while :name cond` only labels the loop.
    m := 0;
    while: spin m < 10 {
        m += 1;
        if m == 2 continue spin;
        if m == 5 break spin;
    }
    assert(m == 5, "labelled while");

    // flow.7: range endpoints are inclusive.
    count := 0;
    for 2..6 count += 1;
    assert(count == 5, "2..6 has five values");
    one := 0;
    for 7..7 one += 1;
    assert(one == 1, "a..a runs once");

    // flow.8: endpoints are evaluated once, left to right.
    array_reset(*trace);
    calls := 0;
    for note(1)..note(3) calls += 1;
    assert(calls == 3, "range ran three times");
    assert(trace.count == 2 && trace[0] == 1 && trace[1] == 3, "endpoints evaluated once, left first");

    // flow.9: default `it` / `it_index`; naming replaces them.
    idx_sum := 0;
    val_sum := 0;
    for arr {
        idx_sum += it_index;
        val_sum += it;
    }
    assert(idx_sum == 3 && val_sum == 60, "it and it_index");
    named_sum := 0;
    for elem, at: arr named_sum += elem + at;
    assert(named_sum == 63, "named value and index");

    // flow.10: plain break/continue target the innermost loop.
    pairs := 0;
    for a: 1..3 {
        for b: 1..3 {
            if b == 2 continue;
            if b == 3 break;
            pairs += 1;
        }
    }
    assert(pairs == 3, "innermost loop only");
    outer_breaks := 0;
    for a: 1..3 {
        outer_breaks += 1;
        for b: 1..3 {
            if b == 2 break a;
        }
    }
    assert(outer_breaks == 1, "break by name leaves the outer loop");

    // flow.11: `remove` is unordered: the last element moves into the hole and is revisited.
    list: [..] int;
    for 1..6 array_add(*list, it);
    for list if it % 2 == 0 remove it;
    assert(list.count == 3 && list[0] == 1 && list[1] == 5 && list[2] == 3, "unordered remove gives [1, 5, 3]");
    bare: [..] int;
    for 1..4 array_add(*bare, it);
    for bare if it == 1 remove;
    assert(bare.count == 3 && bare[0] == 4, "bare remove");

    // flow.12: `for #v2 a..b` is the same inclusive range.
    v2 := 0;
    for #v2 1..4 v2 += it;
    assert(v2 == 10, "#v2 range is inclusive");

    // flow.13: a case runs only its own body unless #through.
    out: [..] string;
    c := Hue.GREEN;
    if c == {
        case .RED; array_add(*out, "red");
        case .GREEN;
            array_add(*out, "green");
            #through;
        case .BLUE; array_add(*out, "blue");
    }
    assert(out.count == 2 && out[0] == "green" && out[1] == "blue", "#through falls into the next body");
    array_reset(*out);
    c = .RED;
    if c == {
        case .RED; array_add(*out, "red");
        case .GREEN; array_add(*out, "green");
    }
    assert(out.count == 1 && out[0] == "red", "no fall through");

    // flow.14: bare `case;` is the default.
    n := 7;
    picked := "";
    if n == {
        case 1; picked = "one";
        case; picked = "default";
    }
    assert(picked == "default", "default label");

    // flow.16: aliases with the same value count as covered by #complete.
    h := Hue.BLUE;
    name := "";
    if #complete h == {
        case .CRIMSON; name = "red";
        case .GREEN; name = "green";
        case .BLUE; name = "blue";
    }
    assert(name == "blue", "alias covers RED");

    // flow.17: #complete on enum_flags or a non-enum is not checked.
    fb := Bits.B;
    flag_hit := false;
    if #complete fb == {
        case .B; flag_hit = true;
    }
    assert(flag_hit, "#complete on enum_flags");
    q := 3;
    if #complete q == {
        case 3; q = 4;
    }
    assert(q == 4, "#complete on an int");

    // flow.18: a constant switch value is not checked.
    CONST_HUE :: Hue.RED;
    const_hit := false;
    if #complete CONST_HUE == {
        case .RED; const_hit = true;
    }
    assert(const_hit, "constant #complete switch");

    // flow.19: defers run at scope exit in reverse order.
    array_reset(*defer_log);
    {
        defer array_add(*defer_log, "d1");
        defer array_add(*defer_log, "d2");
    }
    assert(defer_log.count == 2 && defer_log[0] == "d2" && defer_log[1] == "d1", "reverse order");

    // flow.20: defers also run on return, break and continue.
    array_reset(*defer_log);
    defer_return();
    assert(defer_log.count == 2 && defer_log[0] == "inner" && defer_log[1] == "outer", "defer on return");
    array_reset(*defer_log);
    for 1..3 {
        defer array_add(*defer_log, "it");
        if it == 1 continue;
        if it == 2 break;
    }
    assert(defer_log.count == 2, "defer on continue and break");

    // flow.21: push_context,defer_pop keeps the context for the rest of the block.
    before := context.allocator.data;
    {
        ctx := context;
        ctx.allocator.data = cast(*void) 1234;
        push_context, defer_pop ctx;
        assert(context.allocator.data == cast(*void) 1234, "pushed for the rest of the block");
    }
    assert(context.allocator.data == before, "popped at block end");

    // flow.22: `for x: value` on a type with for_expansion runs the macro.
    down: [..] int;
    for Countdown.{ 3 } array_add(*down, it);
    assert(down.count == 3 && down[0] == 3 && down[2] == 1, "for_expansion iteration");
    skipped := 0;
    for Countdown.{ 4 } {
        if it == 3 continue;
        skipped += it;
    }
    assert(skipped == 4 + 2 + 1, "continue through a for_expansion");

    // flow.23: #insert (break = ...) replaces the body's break.
    got := 0;
    for Halt.{ 10 } {
        if it == 3 break;
        got += 1;
    }
    assert(got == 3 && stopped_by_macro, "replaced break");

    print("ok\n");
}
