// Sans-IO DEC ANSI byte parser. Callback slices are borrowed until the callback returns.
// Example: p: Vt_Parser; vt_feed(*p, "hello\x1b[31m", on_action, user_data);
Vt_Options :: struct {
    max_params: int = 32; // includes colon subparameters and OSC fields, at most 64
    max_param_value: u32 = 65535;
    max_intermediates: int = 2; // at most 8
    max_osc_bytes: int = 4096; // at most 4096, including semicolons
    c1: bool; // raw 8-bit controls; UTF-8 continuation bytes take precedence in ground
}

Vt_Error_Kind :: enum u8 {
    NONE;
    LIMIT;
    OPTIONS;
}

Vt_Error :: struct {
    kind: Vt_Error_Kind;
    message: string;
    offset: u64;
}

Vt_Kind :: enum u8 {
    PRINT;
    EXECUTE;
    CSI_DISPATCH;
    ESC_DISPATCH;
    OSC_DISPATCH;
    DCS_HOOK;
    DCS_PUT;
    DCS_UNHOOK;
}

Vt_Param :: struct {
    value: u32;
    subparam: bool; // preceded by ':' instead of ';'
}

Vt_Action :: struct {
    kind: Vt_Kind;
    code: u32; // codepoint for PRINT, byte/final otherwise
    params: [] Vt_Param;
    intermediates: string;
    private_marker: u8;
    osc: [] string; // split on ';', preserving empty fields
}

Vt_Callback :: (data: *void, action: Vt_Action);

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
