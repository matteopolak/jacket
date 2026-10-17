Console_Style :: enum {
    NORMAL :: 0;
    BOLD :: 1;
    UNDERLINE :: 4;
}

Console_Color :: enum_flags u16 {
    BLACK :: 30;
    RED :: 31;
    GREEN :: 32;
    BLUE :: 34;
    CYAN :: 36;
    MAGENTA :: 35;
    YELLOW :: 33;
    WHITE :: 37;
    HI_BLACK :: 90;
    HI_RED :: 91;
    HI_GREEN :: 92;
    HI_BLUE :: 94;
    HI_CYAN :: 96;
    HI_MAGENTA :: 95;
    HI_YELLOW :: 93;
    HI_WHITE :: 97;
    BG_BLACK :: 40;
    BG_RED :: 41;
    BG_GREEN :: 42;
    BG_BLUE :: 44;
    BG_CYAN :: 46;
    BG_MAGENTA :: 45;
    BG_YELLOW :: 43;
    BG_WHITE :: 47;
    BG_HI_BLACK :: 100;
    BG_HI_RED :: 101;
    BG_HI_GREEN :: 102;
    BG_HI_BLUE :: 104;
    BG_HI_CYAN :: 106;
    BG_HI_MAGENTA :: 105;
    BG_HI_YELLOW :: 103;
    BG_HI_WHITE :: 107;
}

// Foreground and background of a console, plus the two modifiers it can show.
CONSOLE_FOREGROUND_MASK :: 0x000f;
CONSOLE_BACKGROUND_MASK :: 0x00f0;
CONSOLE_FOREGROUND_BRIGHT :: 0x0008;
CONSOLE_BACKGROUND_BRIGHT :: 0x0080;
CONSOLE_UNDERLINE :: 0x8000;

// The console's side of the native engine: read the attribute word of a standard stream and
// replace it. `is_console` is false for a stream that is redirected to a file or pipe (or closed),
// which the engine then leaves alone entirely. Windows installs the real console API; tests (and
// other platforms) install their own.
Console_Backend :: struct {
    query: (to_standard_error: bool) -> (u16, bool);
    apply: (to_standard_error: bool, attributes: u16);
}

// Use `backend` instead of the system console, and forget any colors already pushed.
set_console_backend :: (backend: Console_Backend) {
    native_lock();
    defer native_unlock();
    native_backend = backend;
    native_backend_chosen = true;
    native_streams = .[.{}, .{}];
}

// Go back to the system console.
reset_console_backend :: () {
    set_console_backend(default_console_backend());
}

// How many colors are active on a standard stream (pushed and not yet popped).
console_color_depth :: (to_standard_error := false) -> s64 {
    native_lock();
    defer native_unlock();
    return native_streams[ifx to_standard_error then 1 else 0].depth;
}

// The attribute word that shows `color` and `style` on top of `current`. A foreground color
// replaces only the foreground and a background color only the background, so a block colored
// on its text keeps the enclosing block's background. BOLD brightens the foreground; UNDERLINE
// underlines; NORMAL removes an enclosing underline.
console_attributes_for :: (current: u16, color: Console_Color, style: Console_Style = .NORMAL) -> u16 {
    result := current;
    code := cast(s64) color;
    foreground := code >= 30 && code <= 37 || code >= 90 && code <= 97;
    background := code >= 40 && code <= 47 || code >= 100 && code <= 107;

    if foreground || background {
        index := (code - 30) % 10;
        bits: u16;
        if index & 1 bits |= 0x4;
        if index & 2 bits |= 0x2;
        if index & 4 bits |= 0x1;

        if foreground {
            if code >= 90 bits |= CONSOLE_FOREGROUND_BRIGHT;
            result = (result & ~cast(u16) CONSOLE_FOREGROUND_MASK) | bits;
        } else {
            bits <<= 4;
            if code >= 100 bits |= CONSOLE_BACKGROUND_BRIGHT;
            result = (result & ~cast(u16) CONSOLE_BACKGROUND_MASK) | bits;
        }
    }

    result &= ~cast(u16) CONSOLE_UNDERLINE;
    if style == .BOLD result |= CONSOLE_FOREGROUND_BRIGHT;
    if style == .UNDERLINE result |= CONSOLE_UNDERLINE;

    return result;
}

// Show `color` on a standard stream, remembering what was there. Every push needs a pop; the
// colors nest and each pop restores the enclosing one. A stream that is not a console is not
// touched, but the push still counts so the pops stay balanced.
push_console_color :: (color: Console_Color, style: Console_Style = .NORMAL, to_standard_error := false) {
    native_lock();
    defer native_unlock();

    stream := *native_streams[ifx to_standard_error then 1 else 0];
    level := stream.depth;
    stream.depth += 1;

    backend := current_backend();
    attributes: u16;
    console := false;
    if backend.query attributes, console = backend.query(to_standard_error);

    if level < NATIVE_MAX_DEPTH {
        stream.enclosing[level] = attributes;
        stream.touched[level] = console;
    }

    if !console || !backend.apply return;

    if !stream.has_baseline {
        stream.baseline = attributes;
        stream.has_baseline = true;
    }

    backend.apply(to_standard_error, console_attributes_for(attributes, color, style));
}

// Undo the latest push on a standard stream. The outermost pop returns to the attributes the
// stream had before that push when SAVE_RESTORE_ON_WINDOWS is set, and otherwise to the ones it
// had before the first color was ever pushed. A pop without a push does the latter.
pop_console_color :: (to_standard_error := false) {
    native_lock();
    defer native_unlock();

    backend := current_backend();
    stream := *native_streams[ifx to_standard_error then 1 else 0];

    if stream.depth == 0 {
        if stream.has_baseline && backend.query && backend.apply {
            _, console := backend.query(to_standard_error);
            if console backend.apply(to_standard_error, stream.baseline);
        }

        return;
    }

    stream.depth -= 1;
    level := stream.depth;

    // Levels beyond the table were never recorded; the ones below them restore as usual.
    if level >= NATIVE_MAX_DEPTH || !stream.touched[level] || !backend.apply return;

    restore := stream.enclosing[level];
    if level == 0 && !SAVE_RESTORE_ON_WINDOWS && stream.has_baseline restore = stream.baseline;

    backend.apply(to_standard_error, restore);
}

#if OS != .WINDOWS || USE_ANSI_CODES_ON_WINDOWS {
    set_console_color :: (color: Console_Color, style: Console_Style = .NORMAL, to_standard_error := false) {
        write_string("\e[", to_standard_error);
        write_number(cast(s64) style, to_standard_error = to_standard_error);
        write_string(";", to_standard_error);
        write_number(cast(s64) color, to_standard_error = to_standard_error);
        write_string("m", to_standard_error);
    }

    reset_console_color :: (to_standard_error := false) {
        write_string("\e[0m", to_standard_error);
    }
} else {
    set_console_color :: (color: Console_Color, style: Console_Style = .NORMAL, to_standard_error := false) {
        push_console_color(color, style, to_standard_error);
    }

    reset_console_color :: (to_standard_error := false) {
        pop_console_color(to_standard_error);
    }
}

with_console_color :: (color: Console_Color, code: Code, style: Console_Style = .NORMAL, to_standard_error := false) #expand {
    set_console_color(color, style, to_standard_error);
    defer reset_console_color(to_standard_error);
    #insert code;
}

print_color :: (format_string: string, args: ..Any, color: Console_Color, style: Console_Style = .NORMAL, to_standard_error := false) {
    set_console_color(color, style, to_standard_error);
    defer reset_console_color(to_standard_error);
    B.print(format_string, ..args, to_standard_error = to_standard_error);
} @PrintLike

#scope_file
B :: #import "Basic";

NATIVE_MAX_DEPTH :: 32;

Native_Stream :: struct {
    depth: s64;
    // The attributes before each active level, and whether that level changed the console.
    enclosing: [NATIVE_MAX_DEPTH] u16;
    touched: [NATIVE_MAX_DEPTH] bool;
    // The attributes before the first color ever pushed.
    baseline: u16;
    has_baseline: bool;
}

native_streams: [2] Native_Stream;
native_backend: Console_Backend;
native_backend_chosen: bool;
native_lock_state: s32;

current_backend :: () -> Console_Backend {
    if !native_backend_chosen {
        native_backend = default_console_backend();
        native_backend_chosen = true;
    }

    return native_backend;
}

native_lock :: () {
    acquired := compare_and_swap(*native_lock_state, cast(s32) 0, cast(s32) 1);
    while !acquired acquired = compare_and_swap(*native_lock_state, cast(s32) 0, cast(s32) 1);
}

native_unlock :: () {
    compare_and_swap(*native_lock_state, cast(s32) 1, cast(s32) 0);
}

#if OS == .WINDOWS {
    W :: #import "Windows";

    default_console_backend :: () -> Console_Backend {
        return .{ windows_query, windows_apply };
    }

    windows_handle :: (to_standard_error: bool) -> W.HANDLE {
        handle := W.GetStdHandle(ifx to_standard_error then W.STD_ERROR_HANDLE else W.STD_OUTPUT_HANDLE);
        if !handle || handle == W.INVALID_HANDLE_VALUE return null;
        return handle;
    }

    windows_query :: (to_standard_error: bool) -> (u16, bool) {
        handle := windows_handle(to_standard_error);
        if !handle return 0, false;

        info: W.CONSOLE_SCREEN_BUFFER_INFO;
        if W.GetConsoleScreenBufferInfo(handle, *info) == 0 return 0, false;

        return info.wAttributes, true;
    }

    windows_apply :: (to_standard_error: bool, attributes: u16) {
        handle := windows_handle(to_standard_error);
        if handle W.SetConsoleTextAttribute(handle, attributes);
    }
} else {
    default_console_backend :: () -> Console_Backend {
        return .{};
    }
}
