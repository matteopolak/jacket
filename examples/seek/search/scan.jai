// Looking through one file. What it finds is appended to a buffer as finished output lines, so
// that a worker can scan many files into one buffer and the caller can cut it up afterwards.

// How many bytes at the start of a file are checked for a NUL.
BINARY_PROBE_BYTES :: 8192;

Scan_Result :: struct {
    lines: s64; // lines that matched
    binary: bool; // the file was skipped: it holds a NUL near the start
    error: string; // allocated; empty when the file was read
}

// Whether `text` looks like something that is not text: a NUL in its first 8 KiB.
//
// ```jai
// assert(!looks_binary("plain text\n"));
// assert(looks_binary("PK\x00\x03"));
// ```
looks_binary :: (text: string) -> bool {
    probe := slice(text, 0, min(text.count, BINARY_PROBE_BYTES));

    for probe if it == 0 return true;

    return false;
}

// Searches the file at `path` and appends its output to `buffer`:
//
// - `path:line:col: text` for each line that matches (`path: text` with `no_line_number`),
//   where `col` counts characters from 1 and points at the first match on the line,
// - `path:N` with `count`, for every text file,
// - `path` with `files_only`, for a file with a match.
//
// Binary files add nothing.
scan_file :: (matcher: *Matcher, options: Options, path: string, buffer: *[..] u8) -> Scan_Result {
    result: Scan_Result;

    text, ok := read_entire_file(path);

    if !ok {
        result.error = copy_string(tprint("%: cannot read", path));

        return result;
    }

    defer free(text);

    if looks_binary(text) {
        result.binary = true;

        return result;
    }

    lines: Line_Printer = .{ builder = .{}, path = path, options = options };
    line_number := 0;
    rest := text;

    while rest.count > 0 {
        line := rest;
        newline := find_index_from_left(rest, #char "\n");

        if newline >= 0 {
            line = slice(rest, 0, newline);
            rest = slice(rest, newline + 1, rest.count - newline - 1);
        } else {
            rest = "";
        }

        line_number += 1;

        if ends_with(line, "\r") line.count -= 1;

        at := find_in_line(matcher, line);

        if at < 0 continue;

        result.lines += 1;

        // One line is enough to name the file, so the rest of it need not be read.
        if options.files_only break;

        if !options.count print_match(*lines, line, line_number, at);
    }

    finish_file(*lines, result.lines);
    append_text(buffer, builder_to_string(*lines.builder));

    return result;
}

#scope_file

Line_Printer :: struct {
    builder: String_Builder;
    path: string;
    options: Options;
}

print_match :: (printer: *Line_Printer, line: string, line_number: int, at: int) {
    if printer.options.no_line_number {
        print_to_builder(*printer.builder, "%: %\n", printer.path, line);
    } else {
        print_to_builder(*printer.builder, "%:%:%: %\n", printer.path, line_number, character_column(line, at), line);
    }
}

finish_file :: (printer: *Line_Printer, matches: s64) {
    if printer.options.files_only {
        if matches > 0 print_to_builder(*printer.builder, "%\n", printer.path);
    } else if printer.options.count {
        print_to_builder(*printer.builder, "%:%\n", printer.path, matches);
    }
}

// The 1-based column of byte `at` in `line`, counting characters: continuation bytes of a UTF-8
// sequence do not add to it.
character_column :: (line: string, at: int) -> int {
    column := 1;

    for slice(line, 0, at) {
        if (it & 0xC0) != 0x80 column += 1;
    }

    return column;
}

append_text :: (buffer: *[..] u8, text: string) {
    defer free(text);

    for text array_add(buffer, it);
}
