// A sans-IO parser for terminal output: the DEC ANSI state machine behind escape sequences. Feed it
// bytes in any chunking and it calls you back with printable text, control bytes and CSI, ESC, OSC
// and DCS sequences. A [Vt_Parser] owns nothing, so there is nothing to free. Slices in a
// [Vt_Action] are borrowed until the callback returns.
//
// ```jai
// #import "Extensions/Vt";
//
// count_prints :: (data: *void, action: Vt_Action) {
//     if action.kind != .PRINT return;
//     count := cast(*int) data;
//     count.* += 1;
// }
//
// printed := 0;
// parser: Vt_Parser;
// error := vt_feed(*parser, "hi\x1b[31m!", count_prints, *printed);
// assert(error.kind == .NONE && printed == 3);
// ```

// Limits and modes for a [Vt_Parser]; set them before the first [vt_feed]. A sequence that goes over
// a limit is dropped and reported as `.LIMIT`.
Vt_Options :: struct {
    max_params: int = 32; // includes colon subparameters and OSC fields, at most 64
    max_param_value: u32 = 65535;
    max_intermediates: int = 2; // at most 8
    max_osc_bytes: int = 4096; // at most 4096, including semicolons
    c1: bool; // raw 8-bit controls; UTF-8 continuation bytes take precedence in ground
}

// Why [vt_feed] reported a problem.
Vt_Error_Kind :: enum u8 {
    NONE;
    LIMIT; // a sequence exceeded one of the [Vt_Options] limits and was dropped
    OPTIONS; // the options are out of range or the callback is missing; no bytes were consumed
}

// The outcome of [vt_feed]; `kind` is `.NONE` on success. `message` is static and `offset` counts
// bytes since the parser started.
Vt_Error :: struct {
    kind: Vt_Error_Kind;
    message: string;
    offset: u64;
}

// What a [Vt_Action] reports.
Vt_Kind :: enum u8 {
    PRINT; // a printable codepoint
    EXECUTE; // a C0 or C1 control byte
    CSI_DISPATCH; // a complete control sequence (`ESC [ ...`)
    ESC_DISPATCH; // a complete escape sequence
    OSC_DISPATCH; // a complete operating system command
    DCS_HOOK; // a device control string begins
    DCS_PUT; // one byte of a device control string
    DCS_UNHOOK; // a device control string ends
}

// One numeric parameter of a control sequence.
Vt_Param :: struct {
    value: u32;
    subparam: bool; // preceded by ':' instead of ';'
}

// A parsed event passed to a [Vt_Callback]. Which fields are set depends on `kind`.
Vt_Action :: struct {
    kind: Vt_Kind;
    code: u32; // codepoint for PRINT, byte/final otherwise
    params: [] Vt_Param;
    intermediates: string;
    private_marker: u8;
    osc: [] string; // split on ';', preserving empty fields
}

// Receives each [Vt_Action]; `data` is what you gave [vt_feed]. Do not reenter the parser from it.
Vt_Callback :: (data: *void, action: Vt_Action);

// The parser's position in the escape-sequence grammar, between bytes.
Vt_State :: enum u8 {
    GROUND;
    ESCAPE;
    ESCAPE_INTERMEDIATE;
    CSI_ENTRY;
    CSI_PARAM;
    CSI_INTERMEDIATE;
    CSI_IGNORE;
    DCS_ENTRY;
    DCS_PARAM;
    DCS_INTERMEDIATE;
    DCS_PASSTHROUGH;
    DCS_IGNORE;
    OSC_STRING;
    SOS_PM_APC_STRING;
}

// Zero initialization is ready to feed; no allocation or free is needed.
Vt_Parser :: struct {
    options: Vt_Options;
    state: Vt_State;
    offset: u64;
    params: [64] Vt_Param;
    param_count: int;
    intermediates: [8] u8;
    intermediate_count: int;
    private_marker: u8;
    osc_bytes: [4096] u8;
    osc_count: int;
    osc_fields: [64] string;
    osc_field_count: int = 1;
    dropped: bool;
    utf_code: u32;
    utf_min: u32;
    utf_left: int;
}

// Feeds arbitrary chunks; returns the first limit error in this call while consuming all bytes.
// Example: error := vt_feed(*p, "\x1b[1;2H", on_action, user_data);
// Do not reenter this parser or change its options from a callback. Invalid options consume nothing.
vt_feed :: (p: *Vt_Parser, bytes: string, callback: Vt_Callback, data: *void = null) -> Vt_Error {
    o := p.options;
    if o.max_params < 1 || o.max_params > 64 || o.max_intermediates < 0 || o.max_intermediates > 8 || o.max_osc_bytes < 0 || o.max_osc_bytes > 4096 || !callback return .{ .OPTIONS, "invalid parser options or callback", p.offset };

    error: Vt_Error;

    for bytes {
        failure := step(p, it, callback, data);
        if error.kind == .NONE && failure.kind != .NONE error = failure;
        p.offset += 1;
    }

    return error;
}

// Ends a stream: emits U+FFFD for incomplete UTF-8 and unhooks an active DCS, then resets.
// Example: vt_finish(*p, on_action, user_data); // options and byte offset are retained
vt_finish :: (p: *Vt_Parser, callback: Vt_Callback, data: *void = null) {
    if !callback return;
    if p.utf_left != 0 callback(data, .{ kind = .PRINT, code = 0xfffd });
    if p.state == .DCS_PASSTHROUGH callback(data, .{ kind = .DCS_UNHOOK });

    options := p.options;
    offset := p.offset;

    p.* = .{};
    p.options = options;
    p.offset = offset;
}

#scope_file
#import "Basic";

clear_sequence :: (p: *Vt_Parser) {
    p.param_count = 1;
    p.params[0] = .{};
    p.intermediate_count = 0;
    p.private_marker = 0;
    p.osc_count = 0;
    p.osc_field_count = 1;
    p.dropped = false;
}

limit :: (p: *Vt_Parser) -> Vt_Error {
    p.dropped = true;
    return .{ .LIMIT, "sequence exceeds parser limit", p.offset };
}

sequence :: (p: *Vt_Parser, kind: Vt_Kind, byte: u8, cb: Vt_Callback, data: *void) {
    if p.dropped return;

    a := Vt_Action.{ kind = kind, code = byte, private_marker = p.private_marker };
    if kind == .CSI_DISPATCH || kind == .DCS_HOOK a.params = .{ p.param_count, p.params.data };

    a.intermediates = .{ p.intermediate_count, p.intermediates.data };
    cb(data, a);
}

osc_end :: (p: *Vt_Parser, cb: Vt_Callback, data: *void) {
    if p.dropped return;

    start := 0;
    field := 0;

    for i: 0..p.osc_count {
        if i == p.osc_count || p.osc_bytes[i] == 59 {
            p.osc_fields[field] = .{ i - start, p.osc_bytes.data + start };
            field += 1;
            start = i + 1;
        }
    }

    cb(data, .{ kind = .OSC_DISPATCH, osc = .{ field, p.osc_fields.data } });
}

leave :: (p: *Vt_Parser, cb: Vt_Callback, data: *void, dispatch_osc: bool) {
    if p.state == .DCS_PASSTHROUGH cb(data, .{ kind = .DCS_UNHOOK });
    if p.state == .OSC_STRING && dispatch_osc osc_end(p, cb, data);
    p.state = .GROUND;
}

utf8 :: (p: *Vt_Parser, b: u8, cb: Vt_Callback, data: *void) -> bool {
    if p.utf_left != 0 {
        valid := b >= 0x80 && b <= 0xbf;
        if valid {
            p.utf_code = (p.utf_code << 6) | (b & 0x3f);
            p.utf_left -= 1;
            if p.utf_left == 0 {
                code := p.utf_code;
                if code < p.utf_min || code > 0x10ffff || (code >= 0xd800 && code <= 0xdfff) code = 0xfffd;
                cb(data, .{ kind = .PRINT, code = code });
            }

            return true;
        }

        p.utf_left = 0;
        cb(data, .{ kind = .PRINT, code = 0xfffd });
    }

    return false;
}

step :: (p: *Vt_Parser, b: u8, cb: Vt_Callback, data: *void) -> Vt_Error {
    if p.state == .GROUND && utf8(p, b, cb, data) return .{};

    if b == 0x18 || b == 0x1a {
        leave(p, cb, data, false);
        cb(data, .{ kind = .EXECUTE, code = b });
        return .{};
    }

    if b == 0x1b {
        leave(p, cb, data, true);
        clear_sequence(p);
        p.state = .ESCAPE;
        return .{};
    }

    if p.options.c1 && b >= 0x80 && b <= 0x9f {
        leave(p, cb, data, true);
        clear_sequence(p);
        if b == 0x90 p.state = .DCS_ENTRY;
        else if b == 0x9b p.state = .CSI_ENTRY;
        else if b == 0x9d p.state = .OSC_STRING;
        else if b == 0x98 || b == 0x9e || b == 0x9f p.state = .SOS_PM_APC_STRING;
        else if b != 0x9c cb(data, .{ kind = .EXECUTE, code = b });

        return .{};
    }

    if p.state == .SOS_PM_APC_STRING || p.state == .DCS_IGNORE return .{};

    if p.state == .OSC_STRING {
        if b == 7 {
            osc_end(p, cb, data);
            p.state = .GROUND;
        } else if b >= 32 && !p.dropped {
            if p.osc_count == p.options.max_osc_bytes return limit(p);

            if b == 59 {
                if p.osc_field_count == p.options.max_params return limit(p);
                p.osc_field_count += 1;
            }

            p.osc_bytes[p.osc_count] = b;
            p.osc_count += 1;
        }

        return .{};
    }

    if p.state == .DCS_PASSTHROUGH {
        if b != 127 cb(data, .{ kind = .DCS_PUT, code = b });
        return .{};
    }

    dcs := p.state == .DCS_ENTRY || p.state == .DCS_PARAM || p.state == .DCS_INTERMEDIATE;

    if b < 32 {
        if !dcs cb(data, .{ kind = .EXECUTE, code = b });
        return .{};
    }

    if b == 127 return .{};

    if p.state == .GROUND {
        if b < 128 cb(data, .{ kind = .PRINT, code = b });
        else {
            if b >= 0xc2 && b <= 0xdf {
                p.utf_min = 0x80;
                p.utf_left = 1;
                p.utf_code = b & 0x1f;
            } else if b >= 0xe0 && b <= 0xef {
                p.utf_min = 0x800;
                p.utf_left = 2;
                p.utf_code = b & 0x0f;
            } else if b >= 0xf0 && b <= 0xf4 {
                p.utf_min = 0x10000;
                p.utf_left = 3;
                p.utf_code = b & 7;
            } else cb(data, .{ kind = .PRINT, code = 0xfffd });
        }

        return .{};
    }

    return header_step(p, b, cb, data);
}

header_step :: (p: *Vt_Parser, b: u8, cb: Vt_Callback, data: *void) -> Vt_Error {
    dcs := p.state == .DCS_ENTRY || p.state == .DCS_PARAM || p.state == .DCS_INTERMEDIATE;
    escape := p.state == .ESCAPE || p.state == .ESCAPE_INTERMEDIATE;

    if p.state == .ESCAPE {
        if b == 91 {
            clear_sequence(p);
            p.state = .CSI_ENTRY;
            return .{};
        }

        if b == 80 {
            clear_sequence(p);
            p.state = .DCS_ENTRY;
            return .{};
        }

        if b == 93 {
            clear_sequence(p);
            p.state = .OSC_STRING;
            return .{};
        }

        if b == 88 || b == 94 || b == 95 {
            p.state = .SOS_PM_APC_STRING;
            return .{};
        }
    }

    if p.state == .CSI_IGNORE {
        if b >= 0x40 && b <= 0x7e p.state = .GROUND;
        return .{};
    }

    if b >= 0x20 && b <= 0x2f {
        if p.intermediate_count == p.options.max_intermediates {
            if !escape p.state = ifx dcs then .DCS_IGNORE else .CSI_IGNORE;
            return limit(p);
        }

        p.intermediates[p.intermediate_count] = b;
        p.intermediate_count += 1;
        p.state = ifx escape then .ESCAPE_INTERMEDIATE else ifx dcs then .DCS_INTERMEDIATE else .CSI_INTERMEDIATE;

        return .{};
    }

    if escape {
        if b >= 0x30 && b <= 0x7e {
            sequence(p, .ESC_DISPATCH, b, cb, data);
            p.state = .GROUND;
        }

        return .{};
    }

    entry := p.state == .CSI_ENTRY || p.state == .DCS_ENTRY;
    param := p.state == .CSI_PARAM || p.state == .DCS_PARAM;

    if b >= 0x30 && b <= 0x3f {
        if entry && b >= 0x3c {
            p.private_marker = b;
            p.state = ifx dcs then .DCS_PARAM else .CSI_PARAM;
            return .{};
        }

        if (!entry && !param) || b >= 0x3c {
            p.state = ifx dcs then .DCS_IGNORE else .CSI_IGNORE;
            return .{};
        }

        p.state = ifx dcs then .DCS_PARAM else .CSI_PARAM;

        if b == 58 || b == 59 {
            if p.param_count == p.options.max_params {
                p.state = ifx dcs then .DCS_IGNORE else .CSI_IGNORE;
                return limit(p);
            }

            p.params[p.param_count] = .{ subparam = b == 58 };
            p.param_count += 1;
        } else {
            at := p.param_count - 1;
            value := cast(u64) p.params[at].value * 10 + b - 48;
            p.params[at].value = cast(u32) min(value, cast(u64) p.options.max_param_value);
        }

        return .{};
    }

    if b >= 0x40 && b <= 0x7e {
        sequence(p, ifx dcs then .DCS_HOOK else .CSI_DISPATCH, b, cb, data);
        p.state = ifx dcs then .DCS_PASSTHROUGH else .GROUND;
    }

    return .{};
}
