// Over-aligned types (`#align 64`) through the default allocator: New, alloc, NewArray, array
// growth and resize all return cache-line-aligned blocks for requests of 64 bytes or more, and
// those blocks free normally. Checked at run time and under #run (the interpreter path).
// Prints "ok"; also built natively by crates/jaic-cli/tests/native.rs.
#import "Basic";

Line :: struct {
    member: int #align 64;
}
Two_Lines :: struct {
    first: int #align 64;
    second: int #align 64;
}
Odd :: struct {
    tag: u8;
    payload: [100] u8;
    value: int #align 64;
}

aligned :: (pointer: *void) -> bool {
    return cast(u64) pointer % 64 == 0;
}

check :: () -> string {
    // Interleave sizes so the C heap does not hand out a regular pattern by luck.
    lines: [300] *Line;
    pairs: [300] *Two_Lines;
    odds: [300] *Odd;
    scraps: [300] *void;
    for i: 0..299 {
        scraps[i] = alloc(1 + i % 63);
        lines[i] = New(Line);
        pairs[i] = New(Two_Lines);
        odds[i] = New(Odd);
        if !aligned(lines[i]) return "New(Line)";
        if !aligned(pairs[i]) return "New(Two_Lines)";
        if !aligned(odds[i]) return "New(Odd)";
        lines[i].member = i;
        odds[i].value = i;
    }
    for i: 0..299 {
        if lines[i].member != i || odds[i].value != i return "contents";
        free(lines[i]);
        free(pairs[i]);
        free(odds[i]);
        free(scraps[i]);
    }

    // Every size from a cache line up.
    for size: 64..1100 {
        block := alloc(size);
        if !aligned(block) return "alloc";
        free(block);
    }

    // Growing from below a cache line keeps the bytes and becomes aligned.
    for start: 1..63 {
        block := alloc(start);
        memset(block, 0x5a, start);
        for size: start + 1..start + 300 {
            block = realloc(block, size, size - 1);
            if size >= 64 && !aligned(block) return "realloc";
        }
        for k: 0..start - 1 if (cast(*u8) block)[k] != 0x5a return "realloc contents";
        free(block);
    }

    // Arrays of over-aligned elements, fixed and growing.
    array := NewArray(37, Line);
    if !aligned(array.data) return "NewArray";
    array_free(array);
    growing: [..] Line;
    for i: 0..999 {
        array_add(*growing, .{ member = i });
        if !aligned(growing.data) return "array_add";
    }
    for growing if it.member != it_index return "array contents";
    array_reset(*growing);

    // Locals in the interpreter's frames, and a page-aligned one.
    local: Line;
    Page :: struct {
        member: int #align 4096;
    }
    page: Page;
    if !aligned(*local) return "local";
    if cast(u64) *page % 4096 return "page local";
    return "";
}

#run {
    failure := check();
    if failure print("compile-time failure: %\n", failure);
    assert(failure == "");
}

main :: () {
    failure := check();
    if failure {
        print("failure: %\n", failure);
        exit(1);
    }
    print("ok\n");
}
