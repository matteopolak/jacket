make_help :: (memory: [] u8) -> (success: bool, help: Codex_Help, error_string: string) {
    if memory.count < size_of(Header_Record) return false, .{}, "Codex: short header";

    header := cast(*Header_Record) memory.data;
    if header.magic != 0x05040003 return false, .{}, "Codex: bad magic";
    if header.version != 2 return false, .{}, "Codex: unsupported version";
    if header.num_modules < 0 || header.num_files < 0 || header.num_procedures < 0 return false, .{}, "Codex: negative record count";

    minimum := cast(s64) header.num_modules * size_of(Module_Record) + cast(s64) header.num_files * size_of(File_Record) + cast(s64) header.num_procedures * size_of(Procedure_Record);
    if minimum > memory.count - size_of(Header_Record) return false, .{}, "Codex: truncated record sections";

    help.memory = memory;
    help.header = header;
    cursor := cast(s64) size_of(Header_Record);
    if !codex_check_string(memory, *header.project_name, *cursor) ||
       !codex_check_string(memory, *header.executable_name, *cursor) ||
       !codex_check_string(memory, *header.comment, *cursor) ||
       !codex_check_string(memory, *header.url_or_generation_command, *cursor)
        return false, .{}, "Codex: invalid header strings";

    help.modules = .{ header.num_modules, cast(**Module_Record) alloc(cast(s64) header.num_modules * size_of(*Module_Record)) };
    help.files = .{ header.num_files, cast(**File_Record) alloc(cast(s64) header.num_files * size_of(*File_Record)) };
    help.procedures = .{ header.num_procedures, cast(**Procedure_Record) alloc(cast(s64) header.num_procedures * size_of(*Procedure_Record)) };
    if (help.modules.count > 0 && !help.modules.data) || (help.files.count > 0 && !help.files.data) || (help.procedures.count > 0 && !help.procedures.data) {
        deinit(*help);
        return false, .{}, "Codex: index allocation failed";
    }

    for *entry: help.modules {
        if cursor > memory.count - size_of(Module_Record) {
            deinit(*help);
            return false, .{}, "Codex: truncated module";
        }

        record := cast(*Module_Record) (memory.data + cursor);

        cursor += size_of(Module_Record);
        if !codex_check_string(memory, *record.module_name, *cursor) || !codex_check_string(memory, *record.fully_pathed_filename, *cursor) {
            deinit(*help);
            return false, .{}, "Codex: invalid module strings";
        }

        entry.* = record;
    }

    for *entry: help.files {
        if cursor > memory.count - size_of(File_Record) {
            deinit(*help);
            return false, .{}, "Codex: truncated file";
        }

        record := cast(*File_Record) (memory.data + cursor);

        cursor += size_of(File_Record);
        if !codex_check_string(memory, *record.fully_pathed_filename, *cursor) || record.module_index < 0 || record.module_index >= help.modules.count {
            deinit(*help);
            return false, .{}, "Codex: invalid file record";
        }

        entry.* = record;
    }

    for *entry: help.procedures {
        if cursor > memory.count - size_of(Procedure_Record) {
            deinit(*help);
            return false, .{}, "Codex: truncated procedure";
        }

        record := cast(*Procedure_Record) (memory.data + cursor);

        cursor += size_of(Procedure_Record);
        if !codex_check_string(memory, *record.name, *cursor) || record.file_index < 0 || record.file_index >= help.files.count {
            deinit(*help);
            return false, .{}, "Codex: invalid procedure record";
        }

        entry.* = record;
    }

    return true, help, "";
}

dump_file :: (name: string) -> bool {
    memory, read_ok := read_entire_file(name);
    if !read_ok return false;

    defer free(memory);
    ok, help, error := make_help(.{ memory.count, memory.data });
    if !ok {
        log_error("%\n", error);
        return false;
    }

    defer deinit(*help);
    print("Codex v%: % modules, % files, % procedures\n", help.header.version, help.modules.count, help.files.count, help.procedures.count);
    for record: help.modules print("module % %\n", Relative.decode(*record.module_name), Relative.decode(*record.fully_pathed_filename));
    for record: help.files print("file % module %\n", Relative.decode(*record.fully_pathed_filename), record.module_index);
    for record: help.procedures print("procedure % at %:% calls %\n", Relative.decode(*record.name), record.l0, record.c0, record.metrics.total_calls);

    return true;
}

#scope_file
#import "Basic";
#import "File";
Relative :: #import "Relative_Pointers";

codex_check_string :: (memory: [] u8, field: *Relative.Relative_String(s16), cursor: *s64) -> bool {
    length := cast(s64) field.count;
    if length < 0 || cursor.* >= memory.count || length >= memory.count - cursor.* return false;

    // Compute the relative target as an integer before forming/dereferencing it.
    displacement := cast(s64) field.data;
    if displacement == 0 {
        if length != 0 return false;
    } else {
        displacement ^= -32768;
        field_offset := cast(s64) *field.data - cast(s64) memory.data;
        if field_offset + displacement != cursor.* return false;
    }

    if memory.data[cursor.* + length] != 0 return false;

    cursor.* += length + 1;

    return true;
}
