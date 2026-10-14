main :: () {
    args := get_command_line_arguments();

    defer free(args.data);
    assert(args.count == 2, "provide a caller-owned native font path");
    S.init_fonts();
    S.ensure_fonts_are_initted();
    bytes, ok := File.read_entire_file(args[1], log_errors = false);

    assert(ok);
    defer free(bytes);
    font := S.make_loaded_font("caller-font", .{ bytes.count, bytes.data });

    assert(font != null && font.face != null);
    assert(font.face.num_glyphs > 0 && font.face.units_per_EM > 0);
    assert(font.data.count == bytes.count && font.data.data != bytes.data);
    original := font.data[0];

    bytes[0] ^= 1;
    assert(font.data[0] == original);
    assert(S.find_loaded_font("caller-font") == font);
    S.destroy(font);
    assert(S.find_loaded_font("caller-font") == null);
    invalid := u8.[0, 1, 2, 3];

    assert(S.make_loaded_font("invalid", invalid) == null);
    assert(S.find_loaded_font("invalid") == null);
    file_font := S.make_loaded_font(args[1]);

    assert(file_font != null && file_font.face.num_glyphs > 0);
    S.destroy(file_font);
    bytes[0] = original;
    original_allocator := context.allocator;
    tracking: Font_Test_Allocator;
    tracking.base = original_allocator;
    context.allocator = .{ font_test_allocator, *tracking };
    scoped := S.make_loaded_font("scoped", .{ bytes.count, bytes.data });

    context.allocator = original_allocator;
    persistent := S.make_loaded_font("persistent", .{ bytes.count, bytes.data });

    assert(scoped != null && persistent != null);
    S.destroy(scoped);
    assert(tracking.blocks == 0, "font registry retained the retired caller allocator");
    tracking.active = false;
    S.destroy(persistent);
    print("ok\n");
}

#scope_module
Font_Test_Allocator :: struct {
    base: Allocator;
    active := true;
    blocks: int;
}

font_test_allocator :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    state := cast(*Font_Test_Allocator) allocator_data;

    assert(state.active, "retired allocator called after its last font was destroyed");
    result := state.base.proc(mode, size, old_size, old_memory, state.base.data);
    if (mode == .ALLOCATE || mode == .RESIZE) && old_memory == null && result != null state.blocks += 1;
    if mode == .FREE && old_memory != null state.blocks -= 1;

    return result;
}
