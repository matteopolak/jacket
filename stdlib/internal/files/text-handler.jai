// Reads a text file line by line, skipping comments and blank lines.
//
// Set the options before starting: `comment_character` (`#` by default), `do_version_number` (the
// first line must be a header such as `[3]`), `strip_comments_from_ends_of_lines` and
// `auto_skip_blank_lines`. After starting, `failed` tells whether the source could not be read or the
// version header was bad, `version` is the number from the header (-1 without one) and `line_number` is
// the number of the last line read, counting the comment and blank lines skipped.
//
// A handler that read a file owns the file's bytes. Call `deinit` when done. Lines returned point
// into those bytes and are valid until `deinit` or the next start.
//
// ```jai
// handler: Text_File_Handler;
// handler.do_version_number = false;
// start_from_memory(*handler, "list", "list", "# fruit\napple # red\n\nbanana\n");
// defer deinit(*handler);
//
// line, found := consume_next_line(*handler);
// assert(found && line == "apple" && handler.line_number == 2);
// line, found = consume_next_line(*handler);
// assert(found && line == "banana");
// line, found = consume_next_line(*handler);
// assert(!found);
// ```
Text_File_Handler :: struct {
    short_name: string;
    full_path: string;
    comment_character: u8 = #char "#";
    do_version_number := true;
    strip_comments_from_ends_of_lines := true;
    auto_skip_blank_lines := true;
    file_data_to_free: string;
    file_data: string;
    failed := false;
    version := -1;
    line_number := 0;
}

// Starts reading `_file_data`. `_short_name` and `_full_path` only label the messages the handler reports.
// The text is not copied, so it must outlive the reading; `deinit` does not free it.
//
// Anything the handler held before is released and `failed` is cleared. With `do_version_number` set
// the first line that is not a comment or blank is read as a header `[n]`; if it is missing or is not
// a non-negative number, `failed` is set and an error is logged.
//
// ```jai
// handler: Text_File_Handler;
// start_from_memory(*handler, "config", "config", "[2]\nname jai\n");
// defer deinit(*handler);
// assert(!handler.failed && handler.version == 2);
//
// line, found := consume_next_line(*handler);
// assert(found && line == "name jai");
// ```
start_from_memory :: (using handler: *Text_File_Handler, _short_name: string, _full_path: string, _file_data: string, loc := #caller_location) {
    deinit(handler);
    failed = false;
    short_name = _short_name;
    full_path = _full_path;
    file_data = _file_data;
    version = -1;
    line_number = 0;
    read_version(handler, loc);
}

// Starts reading the file at `full_path`. The file is read whole into memory, owned by the handler and
// freed by `deinit`.
//
// Anything the handler held before is released and `failed` is cleared. When the file cannot be read,
// `failed` is set and an error is logged, unless `optional` is set: then the handler is simply empty and
// does not fail. The version header is checked as in `start_from_memory`.
start_file :: (handler: *Text_File_Handler, short_name: string, full_path: string, optional := false, loc := #caller_location) {
    deinit(handler);
    handler.failed = false;
    handler.short_name = short_name;
    handler.full_path = full_path;
    handler.version = -1;
    handler.line_number = 0;
    bytes, loaded := F.read_entire_file(full_path, log_errors = false);
    if !loaded {
        if !optional {
            handler.failed = true;
            error(handler.*, "cannot read file", loc = loc);
        }

        return;
    }

    handler.file_data_to_free = bytes;
    handler.file_data = bytes;
    read_version(handler, loc);
}

// Frees the file the handler read, if any, and empties it. Safe to call twice. Starting the handler
// again does it too. Lines returned earlier are no longer valid.
deinit :: (using x: *Text_File_Handler) {
    if file_data_to_free.data free(file_data_to_free);
    file_data_to_free = "";
    file_data = "";
} @FreesArgument

// Removes the first line from `sp` and returns it, and whether there was one. A line ends at `\n`, `\r` or
// `\r\n`. Nothing is trimmed or skipped: empty lines and comments come out as they are. At the end of
// the text the result is an empty string and `false`.
//
// ```jai
// text := "one\r\ntwo\n\nthree";
// line, found := consume_next_line(*text);
// assert(found && line == "one");
// line, found = consume_next_line(*text);
// assert(found && line == "two");
// line, found = consume_next_line(*text);
// assert(found && line == "");
// ```
consume_next_line :: (sp: *string) -> string, bool {
    if sp.count <= 0 return "", false;

    count: s64;
    while count < sp.count && sp.*[count] != 10 && sp.*[count] != #char "\r" count += 1;

    line: string = .{ count, sp.data };
    advance := count;

    if advance < sp.count {
        advance += 1;
        if sp.*[count] == #char "\r" && advance < sp.count && sp.*[advance] == 10 advance += 1;
    }

    sp.* = .{ sp.count - advance, sp.data + advance };

    return line, true;
}

// The next line of the handler's text that is not skipped, and whether there was one. Lines whose first non-space
// byte is the comment character are skipped. With `strip_comments_from_ends_of_lines` the line is cut at the first
// comment character, and the line is trimmed of spaces. With `auto_skip_blank_lines`, a line with nothing left
// is skipped too. At the end the result is an empty string and `false`.
//
// `line_number` is advanced for every line looked at, including the ones skipped.
consume_next_line :: (handler: *Text_File_Handler) -> (line: string, found: bool) {
    return processed_line(handler, true);
}

// Splits `line` after its first word: the word, and the rest with the spaces before it removed. Both
// point into `line`. A line of one word gives an empty rest.
//
// ```jai
// word, rest := break_by_spaces("  set  volume 11 ");
// assert(word == "set" && rest == "volume 11");
// ```
break_by_spaces :: (line: string) -> (left: string, right: string) {
    trimmed := S.trim(line);
    index: s64;

    while index < trimmed.count && !space(trimmed[index]) index += 1;

    left: string = .{ index, trimmed.data };

    while index < trimmed.count && space(trimmed[index]) index += 1;

    return left, string.{ trimmed.count - index, trimmed.data + index };
}

// Logs an error for the handler, as `file:line: message`, through the context logger. `format` and `args` work
// as in `print`. The logger receives the `ERROR` flag together with any `flags` given. Nothing is
// written to standard output.
error :: (handler: Text_File_Handler, format: string, args: ..Any, flags := Log_Flags.NONE, loc := #caller_location) {
    report(handler, format, args, flags | .ERROR, loc);
}

// Logs a warning for the handler, as `file:line: message`, through the context logger. See `error`.
warn :: (handler: Text_File_Handler, format: string, args: ..Any, flags := Log_Flags.NONE, loc := #caller_location) {
    report(handler, format, args, flags | .WARNING, loc);
}

// Reads a whole file into an array of lines. Returns whether the file was read and the lines.
//
// Comments are removed, and with `do_trim` each line is trimmed. With `skip_blank_lines` empty lines
// are left out. A version header is expected only if `do_version_number` is set, and then it is not
// among the lines. With `use_data_from_memory` the text in `data_from_memory` is read instead of the file
// (`filename` is then only a label).
//
// When the file cannot be read, or the header is bad, the result is `false` and an empty array, and an
// error is logged. Otherwise the array and every line in it are allocated with the context allocator and
// the caller frees them.
//
// ```jai
// ok, lines := file_to_array("list.txt", use_data_from_memory = true, data_from_memory = "a # one\n\nb\n");
// assert(ok && lines.count == 2 && lines[0] == "a" && lines[1] == "b");
// for line: lines free(line);
// array_free(lines);
// ```
file_to_array :: (filename: string, do_trim := true, skip_blank_lines := true, do_version_number := false, use_data_from_memory := false, data_from_memory := "", loc := #caller_location) -> (success: bool, array: [..] string) {
    handler: Text_File_Handler;
    handler.do_version_number = do_version_number;
    handler.auto_skip_blank_lines = skip_blank_lines;
    if use_data_from_memory start_from_memory(*handler, filename, filename, data_from_memory, loc);
    else start_file(*handler, filename, filename, loc = loc);

    defer deinit(*handler);
    result: [..] string;
    if handler.failed return false, result;

    while true {
        line, found := processed_line(*handler, do_trim);
        if !found break;
        array_add(*result, copy_string(line));
    }

    return true, result;
}

// Like `file_to_array`, but returns the lines as the keys of a table, so repeated lines are kept once. The
// second result is the table. The caller frees each key and the table (`deinit` from Hash_Table). On a
// failure it is `false` and an empty table.
//
// ```jai
// ok, table := file_to_table("set.txt", use_data_from_memory = true, data_from_memory = "x\ny\nx\n");
// assert(ok && table.count == 2);
// ```
file_to_table :: (filename: string, do_trim := true, skip_blank_lines := true, do_version_number := false, use_data_from_memory := false, data_from_memory := "", loc := #caller_location) -> (success: bool, table: T.Table(string, void)) {
    success, lines := file_to_array(filename, do_trim, skip_blank_lines, do_version_number, use_data_from_memory, data_from_memory, loc);

    defer array_free(lines);
    table: T.Table(string, void);
    if !success return false, table;

    unit: void;

    for line: lines {
        if T.table_contains(*table, line) free(line);
        else T.table_add(*table, line, unit);
    }

    return true, table;
}

#scope_file
#import "Basic";
S :: #import "String";
F :: #import "File";
T :: #import "Hash_Table";

space :: (byte: u8) -> bool {
    return byte == #char " " || byte == #char "\t" || byte == #char "\n" || byte == #char "\r";
}

processed_line :: (handler: *Text_File_Handler, trim: bool) -> string, bool {
    while true {
        line, found := consume_next_line(*handler.file_data);
        if !found return "", false;

        handler.line_number += 1;
        significant := S.trim_left(line);
        if significant.count > 0 && significant[0] == handler.comment_character continue;

        if handler.strip_comments_from_ends_of_lines {
            for i: 0..line.count - 1 if line[i] == handler.comment_character {
                line.count = i;
                break;
            }
        }

        if trim line = S.trim(line);
        if handler.auto_skip_blank_lines && S.trim(line).count == 0 continue;

        return line, true;
    }

    return "", false;
}

read_version :: (handler: *Text_File_Handler, loc: Source_Code_Location) {
    if !handler.do_version_number return;

    line, found := consume_next_line(handler);
    if found && line.count >= 3 && line[0] == #char "[" && line[line.count - 1] == #char "]" {
        digits: string = .{ line.count - 2, line.data + 1 };
        value, valid, rest := string_to_int(digits, 10, s64);

        if valid && rest.count == 0 && value >= 0 {
            handler.version = value;
            return;
        }
    }

    handler.failed = true;
    error(handler.*, "invalid or missing version header", loc = loc);
}

report :: (handler: Text_File_Handler, format: string, args: [] Any, flags: Log_Flags, loc: Source_Code_Location) {
    message := sprint(format, ..args);

    defer free(message);
    decorated := sprint("%:%: %\n", handler.full_path, handler.line_number, message);

    defer free(decorated);
    context.logger(decorated, context.logger_data, Log_Info.{ location = loc, common_flags = flags });
}
