compare_single_rewrites :: (a: *Single_Rewrite, b: *Single_Rewrite) -> int {
    if a.l0 != b.l0 return ifx a.l0 < b.l0 then -1 else 1;
    if a.c0 != b.c0 return ifx a.c0 < b.c0 then -1 else 1;
    if a.l1 != b.l1 return ifx a.l1 < b.l1 then -1 else 1;
    if a.c1 != b.c1 return ifx a.c1 < b.c1 then -1 else 1;

    return 0;
}

make_new_file_data :: (my: *My_Plugin, file: *File_To_Rewrite, sorted: [] *Single_Rewrite) -> string {
    result, ok := rewrite_data(file, sorted);
    if !ok {
        log_error("Rewrite: invalid or overlapping source edits for % (write mode %)\n", file.filename, my.write_mode);
        return "";
    }

    return result;
}

rewrite_one_file :: (my: *My_Plugin, file: *File_To_Rewrite) {
    sorted: [..] *Single_Rewrite;

    defer array_free(sorted);
    for *edit: file.single_rewrites array_add(*sorted, edit);

    for index: 1..sorted.count - 1 {
        cursor := index;
        while cursor > 0 && compare_single_rewrites(sorted[cursor - 1], sorted[cursor]) > 0 {
            swap(*sorted[cursor - 1], *sorted[cursor]);
            cursor -= 1;
        }
    }

    result, valid := rewrite_data(file, sorted);
    if !valid {
        log_error("Rewrite: invalid or overlapping source edits for %\n", file.filename);
        return;
    }

    defer free(result);
    for edit: sorted log("Rewrite %:%:%-%:% -> %\n", file.filename, edit.l0, edit.c0, edit.l1, edit.c1, edit.new_text);
    if !my.write_mode || sorted.count == 0 return;

    current, read_ok := read_entire_file(file.filename);

    defer free(current);

    if !read_ok {
        log_error("Rewrite: cannot reread %\n", file.filename);
        return;
    }

    if current != file.full_data {
        log_error("Rewrite: refusing stale source snapshot for %\n", file.filename);
        return;
    }

    if !write_entire_file(file.filename, result) log_error("Rewrite: cannot write %\n", file.filename);
}

init :: (p: *Metaprogram_Plugin, options: [] string) -> bool {
    my := cast(*My_Plugin) p;

    release_rewrites(p);
    my.write_mode = false;

    for option: options {
        if option == "write" my.write_mode = true;
        else {
            log_error("Rewrite: unknown option %\n", option);
            return false;
        }
    }

    my.message = gather;
    my.finish = finish;
    my.shutdown = release_rewrites;

    return true;
}

gather :: (p: *Metaprogram_Plugin, message: *Message) {
    if !message || message.kind != .TYPECHECKED return;

    my := cast(*My_Plugin) p;
    typed := cast(*Message_Typechecked) message;

    for tc: typed.all {
        for node: tc.subexpressions collect_rewrite(my, node);
        collect_rewrite(my, tc.expression);
    }
}

finish :: (p: *Metaprogram_Plugin) {
    my := cast(*My_Plugin) p;
    for *entry: my.files_to_rewrite.entries if entry.hash >= FIRST_VALID_HASH rewrite_one_file(my, *entry.value);
}

#scope_file
#import "Basic";
#import "Compiler";
#import "File";
#import "Hash_Table";
#import "String";

rewrite_offset :: (data: string, line: s32, column: s32) -> s64, bool {
    if line < 1 || column < 1 return 0, false;

    start: s64;
    current: s32 = 1;

    while current < line {
        while start < data.count && data[start] != #char "\n" start += 1;
        if start == data.count return 0, false;
        start += 1;
        current += 1;
    }

    end := start;
    while end < data.count && data[end] != #char "\n" end += 1;

    offset := start + cast(s64) column - 1;

    return offset, offset <= end;
}

rewrite_data :: (file: *File_To_Rewrite, sorted: [] *Single_Rewrite) -> string, bool {
    builder: String_Builder;

    defer free_buffers(*builder);
    previous_end: s64;
    previous_start: s64 = -1;

    for edit: sorted {
        if !edit return "", false;

        start, valid_start := rewrite_offset(file.full_data, edit.l0, edit.c0);
        end, valid_end := rewrite_offset(file.full_data, edit.l1, edit.c1);

        if !valid_start || !valid_end || start < previous_end || start == previous_start || end < start return "", false;

        append(*builder, slice(file.full_data, previous_end, start - previous_end));
        append(*builder, edit.new_text);
        previous_start = start;
        previous_end = end;
    }

    append(*builder, slice(file.full_data, previous_end, file.full_data.count - previous_end));
    if builder.failed return "", false;

    return builder_to_string(*builder), true;
}

collect_rewrite :: (my: *My_Plugin, node: *Code_Node) {
    if !node || node.kind != .UNARY_OPERATOR || !node.enclosing_load || node.enclosing_load.from_a_string return;

    unary := cast(*Code_Unary_Operator) node;
    if unary.operator_type != .POINTER_DEREFERENCE && unary.operator_type != .POSTFIX_DEREFERENCE return;
    if !unary.subexpression return;

    file := table_find_pointer(*my.files_to_rewrite, node.enclosing_load);
    if !file {
        snapshot: File_To_Rewrite;
        snapshot.filename = copy_string(node.enclosing_load.fully_pathed_filename);
        ok: bool;
        snapshot.full_data, ok = read_entire_file(snapshot.filename);
        if !ok {
            log_error("Rewrite: cannot read %\n", snapshot.filename);
            free(snapshot.filename);
            return;
        }

        snapshot.lines = split(snapshot.full_data, "\n");
        file = table_add(*my.files_to_rewrite, node.enclosing_load, snapshot);
    }

    start, start_ok := rewrite_offset(file.full_data, node.l0, node.c0);
    end, end_ok := rewrite_offset(file.full_data, node.l1, node.c1);
    child_start, child_start_ok := rewrite_offset(file.full_data, unary.subexpression.l0, unary.subexpression.c0);
    child_end, child_end_ok := rewrite_offset(file.full_data, unary.subexpression.l1, unary.subexpression.c1);

    if !start_ok || !end_ok || !child_start_ok || !child_end_ok || end < start || child_start < start || child_end > end {
        log_error("Rewrite: invalid compiler source locations for %\n", file.filename);
        return;
    }

    if end - start < 2 || slice(file.full_data, start, 2) != "<<" return;
    for edit: file.single_rewrites if edit.l0 == node.l0 && edit.c0 == node.c0 && edit.l1 == node.l1 && edit.c1 == node.c1 return;

    replacement := sprint("(%).*", slice(file.full_data, child_start, child_end - child_start));

    array_add(*file.single_rewrites, Single_Rewrite.{ node.l0, node.c0, node.l1, node.c1, replacement });
}

release_rewrites :: (p: *Metaprogram_Plugin) {
    my := cast(*My_Plugin) p;
    for entry: my.files_to_rewrite.entries if entry.hash >= FIRST_VALID_HASH {
        for edit: entry.value.single_rewrites free(edit.new_text);

        array_free(entry.value.single_rewrites);
        free(entry.value.lines.data);
        free(entry.value.full_data);
        free(entry.value.filename);
    }

    deinit(*my.files_to_rewrite);
}
