// All generator state expires within the same compile-time invocation.
// jailint: allow-file(temp_storage_escape)

// Implementation entry points used by generated templates.
// ```jai
// out: Out;
// defer deinit(*out);
// Template.value(*out, 12, false, false, false, "12");
// assert(to_string(*out) == "12");
// ```
Template :: struct {
    // Starts recording one link; used by generated templates.
    // ```jai
    // out: Out;
    // defer deinit(*out);
    // Template.begin(*out);
    // Template.piece(*out, "/");
    // Template.finish(*out);
    // assert(out.links[0] == "/");
    // ```
    begin :: (out: *Out) {
        out.link.count = 0;
    }
    // Records unescaped link bytes when recording is enabled.
    // ```jai
    // out: Out;
    // defer deinit(*out);
    // Template.begin(*out);
    // Template.piece(*out, "/");
    // Template.finish(*out);
    // assert(out.links[0] == "/");
    // ```
    piece :: (out: *Out, s: string) {
        if out.record_links array_add(*out.link, ..(cast([] u8) s));
    }
    // Saves the recorded link into the page arena.
    // ```jai
    // out: Out;
    // defer deinit(*out);
    // Template.begin(*out);
    // Template.piece(*out, "/");
    // Template.finish(*out);
    // assert(out.links[0] == "/");
    // ```
    finish :: (out: *Out) {
        prepare(out);
        if out.record_links array_add(*out.links, save(*out.pool, cast(string) out.link));
    }
    // Appends a typed template hole; unsupported types report at location.
    // ```jai
    // out: Out;
    // defer deinit(*out);
    // Template.value(*out, "<&", false, false, false, "text");
    // assert(to_string(*out) == "&lt;&amp;");
    // ```
    value :: (out: *Out, v: $T, $attribute: bool, $raw: bool, $link: bool, $hole: string, $location: Source_Code_Location = #caller_location) {
        #if T == Html {
            append_html(out, v);
            #if link piece(out, cast(string) v);
        } else #if raw {
            #run compiler_report(tprint("Web: hole `{%}` has type %; script/style holes require Html", hole, T), location);
        } else #if T == string {
            #if attribute escape_attribute(out, v);
            else escape_text(out, v);
            #if link piece(out, v);
        } else #if type_info(T).type == .INTEGER {
            text := tprint("%", v);
            append(out, text);
            #if link piece(out, text);
        } else {
            #run compiler_report(tprint("Web: hole `{%}` has unsupported type %", hole, T), location);
        }
    }
}

#scope_module

Mode :: enum {
    TEXT;
    TAG;
    NAME;
    BETWEEN;
    ATTRIBUTE;
    AFTER_ATTRIBUTE;
    VALUE;
    QUOTED;
    UNQUOTED;
    COMMENT;
    RAW;
}

Scan :: struct {
    mode: Mode;
    quote: u8;
    name: string;
    attribute: string;
    closing: bool;
    self_closing: bool;
    link: bool;
    tags: [..] string;
}

Branch :: struct {
    entry: string;
    entry_state: Scan;
    exit: string;
    is_if: bool;
    has_else: bool;
}

Generator :: struct {
    code: String_Builder;
    output: string;
    module_name: string;
    literal: String_Builder;
    scan: Scan;
    branches: [..] Branch;
    node: *Code_Literal;
    location: Source_Code_Location;
    failed: bool;
}

quoted :: (s: string) -> string {
    b: String_Builder;

    B.append(*b, #char "\"");

    for c: s {
        if c == #char "\"" B.append(*b, "\\\"");
        else if c == #char "\\" B.append(*b, "\\\\");
        else if c == #char "\n" B.append(*b, "\\n");
        else if c == #char "\r" B.append(*b, "\\r");
        else if c == #char "\t" B.append(*b, "\\t");
        else if c < 32 print_to_builder(*b, "\\x%", formatInt(c, base = 16, minimum_digits = 2));
        else B.append(*b, c);
    }

    B.append(*b, #char "\"");

    return builder_to_string(*b,, temp);
}

fail :: (g: *Generator, offset: int, message: string) {
    if g.failed return;

    g.failed = true;
    location := g.location;
    if g.node location, _ = Literal.location_of_byte(g.node, offset);

    compiler_report(tprint("Web: %", message), location);
}

flush :: (g: *Generator) {
    text := builder_to_string(*g.literal,, temp);
    if text.count == 0 return;
    print_to_builder(*g.code, "%.append(%, %);\n", g.module_name, g.output, quoted(text));
    if g.scan.link print_to_builder(*g.code, "%.Template.piece(%, %);\n", g.module_name, g.output, quoted(text));
}

signature :: (s: *Scan) -> string {
    b: String_Builder;
    print_to_builder(*b, "%/%/%/%/%/", s.mode, s.quote, s.name, s.attribute, s.link);
    for s.tags print_to_builder(*b, "%/", it);
    return builder_to_string(*b,, temp);
}

space :: (c: u8) -> bool {
    return c == 32 || c == 9 || c == 10 || c == 13;
}

word :: (c: u8) -> bool {
    return (c >= 65 && c <= 90) || (c >= 97 && c <= 122) || (c >= 48 && c <= 57) || c == 45 || c == 58 || c == 95;
}

void_tag :: (s: string) -> bool {
    for string.["area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr"] if it == s return true;
    return false;
}

close_tag :: (g: *Generator, at: int) {
    s := *g.scan;
    if s.closing {
        if s.tags.count == 0 || s.tags[s.tags.count - 1] != s.name {
            fail(g, at, tprint("unbalanced closing tag </%>", s.name));
        } else s.tags.count -= 1;
    } else if !s.self_closing && !void_tag(s.name) && !begins_with(s.name, "!") {
        array_add(*s.tags, s.name);
    }

    s.mode = .TEXT;
    if !s.closing && !s.self_closing && (s.name == "script" || s.name == "style") s.mode = .RAW;

    s.name = "";
    s.attribute = "";
    s.closing = false;
    s.self_closing = false;
}

// Scans only literals. Every transition that begins/ends a link emits recording code.
scan_byte :: (g: *Generator, text: string, i: int) {
    s := *g.scan;
    c := text[i];

    if s.mode == .COMMENT {
        B.append(*g.literal, c);
        if i >= 2 && slice(text, i - 2, 3) == "-->" s.mode = .TEXT;
        return;
    }

    if s.mode == .RAW {
        if !raw_close(text, i, s.tags[s.tags.count - 1]) {
            B.append(*g.literal, c);
            return;
        }

        s.mode = .TEXT;
    }

    if s.mode == .TEXT {
        if c == #char "<" {
            if begins_with(advance(text, i), "<!--") s.mode = .COMMENT;
            else s.mode = .TAG;
        }
    } else if s.mode == .TAG {
        if c == #char "/" s.closing = true;
        else {
            s.name = lower(slice(text, i, 1));
            s.mode = .NAME;
        }
    } else if s.mode == .NAME {
        if word(c) || c == #char "!" s.name = tprint("%%", s.name, lower(slice(text, i, 1)));
        else if c == #char ">" close_tag(g, i);
        else if c == #char "/" {
            s.self_closing = true;
            s.mode = .BETWEEN;
        } else s.mode = .BETWEEN;
    } else if s.mode == .BETWEEN {
        if c == #char ">" close_tag(g, i);
        else if c == #char "/" s.self_closing = true;
        else if !space(c) {
            s.attribute = lower(slice(text, i, 1));
            s.mode = .ATTRIBUTE;
        }
    } else if s.mode == .ATTRIBUTE {
        if c == #char "=" s.mode = .VALUE;
        else if space(c) s.mode = .AFTER_ATTRIBUTE;
        else if c == #char ">" close_tag(g, i);
        else s.attribute = tprint("%%", s.attribute, lower(slice(text, i, 1)));
    } else if s.mode == .AFTER_ATTRIBUTE {
        if c == #char "=" s.mode = .VALUE;
        else if c == #char ">" close_tag(g, i);
        else if !space(c) {
            s.attribute = lower(slice(text, i, 1));
            s.mode = .ATTRIBUTE;
        }
    } else if s.mode == .VALUE {
        if c == #char "\"" || c == #char "'" {
            s.quote = c;
            s.mode = .QUOTED;
            recording := s.attribute == "href" || s.attribute == "src";
            if recording {
                B.append(*g.literal, c);
                flush(g);
                s.link = true;
                print_to_builder(*g.code, "%.Template.begin(%);\n", g.module_name, g.output);

                return;
            }
        } else if !space(c) s.mode = .UNQUOTED;
    } else if s.mode == .QUOTED {
        if c == s.quote {
            if s.link {
                flush(g);
                print_to_builder(*g.code, "%.Template.finish(%);\n", g.module_name, g.output);
            }

            s.link = false;
            s.quote = 0;
            s.attribute = "";
            s.mode = .BETWEEN;
        }
    } else if s.mode == .UNQUOTED {
        if c == #char ">" close_tag(g, i);
        else if space(c) {
            s.mode = .BETWEEN;
            s.attribute = "";
        }
    }

    B.append(*g.literal, c);
}

// Finds the matching brace, respecting Jai strings and nested expression braces.
hole_end :: (t: string, start: int) -> int {
    depth := 0;
    quote: u8;
    i := start;

    while i < t.count {
        c := t[i];

        if quote != 0 {
            if c == #char "\\" {
                i += 2;
                continue;
            }

            if c == quote quote = 0;
        } else if c == #char "\"" || c == #char "'" quote = c;
        else if c == #char "{" depth += 1;
        else if c == #char "}" {
            if depth == 0 return i;
            depth -= 1;
        }

        i += 1;
    }

    return -1;
}

control :: (g: *Generator, h: string, at: int) -> bool {
    is_if := begins_with(h, "if ");
    is_for := begins_with(h, "for ");
    is_else := h == "else" || begins_with(h, "else if ");

    if !is_if && !is_for && !is_else && h != "end" return false;

    s := *g.scan;
    // A tag name immediately before a control has finished, e.g. <a{if x} ...
    if s.mode == .NAME s.mode = .BETWEEN;

    if s.mode != .TEXT && s.mode != .BETWEEN {
        fail(g, at, "control flow is only allowed in text or between attributes");
        return true;
    }

    if is_if || is_for {
        array_add(*g.branches, .{ entry = signature(s), entry_state = snapshot(s), is_if = is_if });
        print_to_builder(*g.code, "% {\n", h);
    } else {
        if g.branches.count == 0 {
            fail(g, at, ifx is_else then "else outside if" else "end with nothing open");
            return true;
        }

        branch := *g.branches[g.branches.count - 1];

        if is_else {
            if !branch.is_if || branch.has_else {
                fail(g, at, "else outside if");
                return true;
            }

            exit := signature(s);
            if branch.exit.count && branch.exit != exit fail(g, at, "HTML state differs across if branches");

            branch.exit = exit;
            g.scan = snapshot(*branch.entry_state);

            branch.has_else = h == "else";
            print_to_builder(*g.code, "} % {\n", h);
        } else {
            exit := signature(s);
            if branch.exit.count && exit != branch.exit fail(g, at, "unbalanced tags across if branches");
            if (!branch.is_if || !branch.has_else) && exit != branch.entry fail(g, at, "unbalanced tags across control flow");

            g.branches.count -= 1;
            B.append(*g.code, "}\n");
        }
    }

    return true;
}

compile_template :: (t: string, loc: Code) -> string {
    g: Generator;
    root := compiler_get_nodes(loc);

    g.location = .{ fully_pathed_filename = root.enclosing_load.fully_pathed_filename, line_number = root.l0, character_number = root.c0 };
    call := cast(*Code_Procedure_Call) root;
    args := call.arguments_sorted;

    if args.count < 2 {
        fail(*g, 0, "cannot inspect html arguments");
        return "";
    }

    dest := args[0];
    output := "";

    if dest.kind == .IDENT output = (cast(*Code_Ident) dest).name;
    else if dest.kind == .UNARY_OPERATOR {
        unary := cast(*Code_Unary_Operator) dest;
        if unary.subexpression.kind == .IDENT output = tprint("*%", (cast(*Code_Ident) unary.subexpression).name);
    }

    if output.count == 0 {
        fail(*g, 0, "output must be an identifier or *identifier");
        return "";
    }

    if args[1].kind == .LITERAL g.node = cast(*Code_Literal) args[1];

    g.output = output;
    g.module_name = "__web";
    while contains(t, g.module_name) || contains(output, g.module_name) g.module_name = tprint("%_", g.module_name);

    print_to_builder(*g.code, "{ % :: #import \"Extensions/Web\";\n", g.module_name);
    i := 0;
    while i < t.count && !g.failed {
        if t[i] != #char "{" {
            scan_byte(*g, t, i);
            i += 1;
            continue;
        }

        if i + 1 < t.count && t[i + 1] == #char "{" {
            B.append(*g.literal, #char "{");
            i += 2;
            continue;
        }

        flush(*g);
        end := hole_end(t, i + 1);
        if end < 0 {
            fail(*g, i, "unclosed {");
            break;
        }

        h := trim(slice(t, i + 1, end - i - 1));
        if !control(*g, h, i) {
            paren := find_index_from_left(h, #char "(");
            component := h.count > 0 && h[0] >= 65 && h[0] <= 90 && paren > 0;

            if component {
                for j: 0..paren - 1 if !word(h[j]) component = false;
            }

            if component {
                if g.scan.mode != .TEXT {
                    fail(*g, i, "component call inside tag or raw text");
                    break;
                }

                args_text := slice(h, paren + 1, h.count - paren - 2);

                print_to_builder(*g.code, "%(% % %);\n", slice(h, 0, paren), g.output, ifx trim(args_text).count then "," else "", args_text);
            } else {
                mode := g.scan.mode;
                if mode != .TEXT && mode != .QUOTED && mode != .RAW {
                    fail(*g, i, "value hole outside text or quoted attribute");
                    break;
                }

                location := g.location;
                if g.node location, _ = Literal.location_of_byte(g.node, i);

                print_to_builder(*g.code, "%.Template.value(%, (%), %, %, %, %, .{fully_pathed_filename = %, line_number = %, character_number = %});\n", g.module_name, g.output, h, mode == .QUOTED, mode == .RAW, g.scan.link, quoted(h), quoted(location.fully_pathed_filename), location.line_number, location.character_number);
            }
        }

        i = end + 1;
    }

    flush(*g);
    if g.branches.count fail(*g, t.count, "missing end");
    if g.scan.tags.count || g.scan.mode != .TEXT fail(*g, t.count, "unbalanced HTML tags");

    B.append(*g.code, "}\n");

    return builder_to_string(*g.code,, temp);
}

// HTML element and attribute names are ASCII case-insensitive.
lower :: (s: string) -> string {
    result := copy_temporary_string(s);
    for *result if it.* >= 65 && it.* <= 90 it.* += 32;
    return result;
} @TempAllocates

raw_close :: (text: string, i: int, tag: string) -> bool {
    if !begins_with(advance(text, i), "</") return false;
    end := i + 2 + tag.count;
    if end >= text.count return false;
    return lower(slice(text, i + 2, tag.count)) == tag && (space(text[end]) || text[end] == #char ">");
}

snapshot :: (s: *Scan) -> Scan {
    result := s.*;
    result.tags = .{};
    result.tags.allocator = temp;
    for s.tags array_add(*result.tags, it);

    return result;
}
