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

start_from_memory :: (using handler: *Text_File_Handler, _short_name: string, _full_path: string, _file_data: string, loc := #caller_location) {
    deinit(handler);
    short_name = _short_name;
    full_path = _full_path;
    file_data = _file_data;
    version = -1;
    line_number = 0;
    read_version(handler, loc);
}

start_file :: (handler: *Text_File_Handler, short_name: string, full_path: string, optional := false, loc := #caller_location) {
    deinit(handler);
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

deinit :: (using x: *Text_File_Handler) {
    if file_data_to_free.data free(file_data_to_free);
    file_data_to_free = "";
    file_data = "";
} @Releases

consume_next_line :: (sp: *string) -> string, bool {
    if sp.count <= 0 return "", false;

    count: s64;
    while count < sp.count && sp.*[count] != 10 && sp.*[count] != 13 count += 1;

    line: string = .{ count, sp.data };
    advance := count;

    if advance < sp.count {
        advance += 1;
        if sp.*[count] == 13 && advance < sp.count && sp.*[advance] == 10 advance += 1;
    }

    sp.* = .{ sp.count - advance, sp.data + advance };

    return line, true;
}

consume_next_line :: (handler: *Text_File_Handler) -> (line: string, found: bool) {
    return processed_line(handler, true);
}

break_by_spaces :: (line: string) -> (left: string, right: string) {
    trimmed := S.trim(line);
    index: s64;

    while index < trimmed.count && !space(trimmed[index]) index += 1;

    left: string = .{ index, trimmed.data };

    while index < trimmed.count && space(trimmed[index]) index += 1;

    return left, string.{ trimmed.count - index, trimmed.data + index };
}

error :: (handler: Text_File_Handler, format: string, args: ..Any, flags := Log_Flags.NONE, loc := #caller_location) {
    report(handler, format, args, flags | .ERROR, loc);
}

warn :: (handler: Text_File_Handler, format: string, args: ..Any, flags := Log_Flags.NONE, loc := #caller_location) {
    report(handler, format, args, flags | .WARNING, loc);
}

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
    return byte == 32 || byte == 9 || byte == 10 || byte == 13;
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
    if found && line.count >= 3 && line[0] == 91 && line[line.count - 1] == 93 {
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
