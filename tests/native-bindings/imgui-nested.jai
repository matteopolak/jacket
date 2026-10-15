// Real Jai-to-C++ calls, with native header ABI checks before constructors.
G :: #import "ImGui";

probe :: #library,system "imgui_abi_probe";
native_size :: (record: u32) -> u64 #foreign probe "binding_imgui_size";
native_offset :: (field: u32) -> u64 #foreign probe "binding_imgui_offset";
libc :: #library,system "c";
abort :: () #foreign libc;

check :: (value: bool) {
    if !value abort();
}

main :: () {
    check(size_of(G.TextFilter) == native_size(0));
    check(size_of(G.TextFilter.TextRange) == native_size(1));
    check(size_of(G.ImVector(G.TextFilter.TextRange)) == native_size(2));
    check(offset_of(G.TextFilter, "InputBuf") == native_offset(0));
    check(offset_of(G.TextFilter, "Filters") == native_offset(1));
    check(offset_of(G.TextFilter, "CountGrep") == native_offset(2));
    check(offset_of(G.TextFilter.TextRange, "e") == native_offset(3));
    check(offset_of(G.ImVector(G.TextFilter.TextRange), "Data") == native_offset(4));
    check(type_info(G.TextFilter.TextRange).name == "TextRange");
    context_ := G.CreateContext();

    check(context_ != null);
    defer G.DestroyContext(context_);
    source, destination: G.TextFilter;

    G.TextFilter.Constructor(*source, "one,two,three");
    G.TextFilter.Constructor(*destination, "");
    defer G.MemFree(source.Filters.Data);
    defer G.MemFree(destination.Filters.Data);
    check(source.Filters.Size == 3);
    // Use SDK-constructed objects, preserving their C++ initialization/lifetime
    // invariants rather than treating arbitrary byte storage as a live vector.
    range := source.Filters.Data;
    range.b = source.InputBuf.data;
    range.e = source.InputBuf.data + 13;
    G.TextFilter.TextRange.split(range, #char ",", *destination.Filters);
    check(destination.Filters.Size == 3);
    expected: [3] string = .["one", "two", "three"];

    for i: 0..2 {
        part := destination.Filters.Data[i];
        text: string = .{ data = part.b, count = cast(s64) (cast(u64) part.e - cast(u64) part.b) };
        check(text == expected[i]);
    }
}
