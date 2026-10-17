B :: #import "Basic";
Off :: #import "Basic"()(ENABLE_ASSERT = false);
Other :: #import "Basic"()(VISUALIZE_MEMORY_DEBUGGER = false);

Record :: struct {
    number: int;
}

backing: Allocator;
live: int;
calls: int;

tracked :: (mode: Allocator_Mode, size: s64, old_size: s64, memory: *void, data: *void) -> *void {
    result := backing.proc(mode, size, old_size, memory, backing.data);
    if (mode == .ALLOCATE || (mode == .RESIZE && !memory)) && result live += 1;
    if mode == .FREE && memory live -= 1;
    return result;
}

custom :: (builder: *B.String_Builder, item: Any, data: *void) -> bool {
    _ := item;

    B.assert(data == *calls);
    calls += 1;
    prefix := B.builder_to_string(builder, do_reset = false);

    B.assert(prefix == "prefix:");
    B.free(prefix);
    for 1..5000 B.append(builder, #char "x");

    return true;
}

declining :: (builder: *B.String_Builder, _item: Any, _data: *void) -> bool {
    B.append(builder, "!");
    return false;
}

main :: () {
    backing = context.allocator;
    B.assert(B.Print_Style == Off.Print_Style && B.Print_Style == Other.Print_Style);
    B.assert(B.FormatInt == Off.FormatInt);
    B.assert((cast(*Type_Info_Struct) type_info(B.Print_Style)).name == "Print_Style");
    B.assert((cast(*Type_Info_Struct) type_info(B.Formatter)).name == "Formatter");
    B.assert(size_of(B.String_Builder.Buffer) != size_of(Off.String_Builder.Buffer));
    info := cast(*Type_Info_Struct) type_info(#Context);
    fields := 0;

    for info.members if it.name == "print_style" fields += 1;

    B.assert(fields == 1);
    context.print_style.default_format_int.base = 16;
    context.print_style.default_format_array.begin_string = "<";
    context.print_style.default_format_array.end_string = ">";
    text := Other.sprint("%", 255);

    B.assert(text == "ff");
    B.free(text);
    text = Off.sprint("%", int.[10, 11]);
    B.assert(text == "<a, b>");
    Off.free(text);
    formatter := Other.formatInt(31, base = 2, minimum_digits = 8);

    text = Off.sprint("%", formatter);
    B.assert(text == "00011111");
    Off.free(text);
    context.print_style.default_format_float.mode = .SHORTEST;
    text = Other.sprint("%", cast(float64) 1.25);
    B.assert(text == "1.25");
    B.free(text);

    {
        B.push_allocator(tracked, null);
        context.print_style.struct_printer = custom;
        context.print_style.struct_printer_data = *calls;
        record := Record.{ 7 };
        first := B.sprint("prefix:%", record);
        second := Off.sprint("prefix:%", record);

        B.assert(first == second && first.count == 5007 && calls == 2);
        B.free(first);
        Off.free(second);
        context.print_style.struct_printer = declining;
        first = B.sprint("prefix:%", record);
        second = Off.sprint("prefix:%", record);
        B.assert(first == second && first.count > 8);
        B.free(first);
        Off.free(second);
        context.print_style.struct_printer = null;
    }

    B.assert(live == 0);
    // Context copies carry the same declared field with independent value storage.
    copied := context;
    copied.print_style.default_format_int.base = 10;
    push_context copied {
        text = Off.sprint("%", 255);
        Off.assert(text == "255");
        Off.free(text);
    }

    B.assert(context.print_style.default_format_int.base == 16);
}
