#scope_module
// IMM32 positioning and decoding belong to this extension.
set_rect :: (window: W.HWND, enabled: bool, x: float, y: float, height: float) {
    imc := get_context(window);
    if !imc return;

    defer release_context(window, imc);

    if !enabled {
        notify(imc, 0x15, 4, 0); // NI_COMPOSITIONSTR, CPS_CANCEL
        return;
    }

    form: Composition_Form;
    form.style = 2; // CFS_POINT, in client coordinates
    form.position = .{ cast(s32) x, cast(s32) (y + height) };
    set_composition_window(imc, *form);
}

read_text :: (window: W.HWND, index: u32) -> string {
    imc := get_context(window);
    if !imc return "";

    defer release_context(window, imc);
    bytes := get_composition(imc, index, null, 0);
    if bytes <= 0 || bytes % 2 != 0 return "";

    Basic.push_allocator(context.default_allocator);
    units := Basic.NewArray(bytes / 2, u16);

    defer Basic.array_free(units);
    if get_composition(imc, index, units.data, cast(u32) bytes) != bytes return "";

    text, ok := Utf8.wide_to_utf8(units.data, cast(s32) units.count);
    if !ok return "";

    return text;
}

is_enabled :: (window: W.HWND) -> bool {
    for hook: window_hooks if hook.window == window return hook.enabled;
    return false;
}

// Convert IMM32's UTF-16 cursor to the UTF-8 byte offset carried by Input events.
cursor_offset :: (window: W.HWND, text: string) -> int {
    imc := get_context(window);
    if !imc return text.count;

    defer release_context(window, imc);
    cursor := get_composition(imc, 0x80, null, 0); // GCS_CURSORPOS returns the position directly.
    if cursor <= 0 return 0;

    Basic.push_allocator(context.default_allocator);
    wide, ok, bytes := Utf8.utf8_to_wide(text);
    if !ok return text.count;

    defer Basic.free(wide);
    units := Basic.min(cursor, bytes / 2);
    if units > 0 && units < bytes / 2 && wide[units - 1] >= 0xd800 && wide[units - 1] <= 0xdbff units -= 1;
    if units == 0 return 0;

    prefix, converted := Utf8.wide_to_utf8(wide, units);
    if !converted return text.count;

    defer Basic.free(prefix);

    return prefix.count;
}

#scope_module
Utf8 :: #import "Windows_Utf8";

Composition_Form :: struct {
    style: u32;
    position: W.POINT;
    area: W.RECT;
}

imm32 :: #library,system "imm32";
get_context :: (window: W.HWND) -> *void #foreign imm32 "ImmGetContext";
release_context :: (window: W.HWND, imc: *void) -> s32 #foreign imm32 "ImmReleaseContext";
get_composition :: (imc: *void, index: u32, buffer: *void, size: u32) -> s32 #foreign imm32 "ImmGetCompositionStringW";
set_composition_window :: (imc: *void, form: *Composition_Form) -> s32 #foreign imm32 "ImmSetCompositionWindow";
notify :: (imc: *void, action: u32, index: u32, value: u32) -> s32 #foreign imm32 "ImmNotifyIME";
