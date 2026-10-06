// File: open modes, reading through writing handles, appending, seeking, lengths, builders,
// zero termination, mapping, hard links, empty and multi-megabyte files, non-ASCII names, and
// clean failures for missing files and bad arguments.
#import "Basic";
#import "File";
#import "String";

ROOT :: ".scratch/file-handles";

main :: () {
    delete_directory(ROOT); // leftovers of an earlier failed run
    assert(make_directory_if_it_does_not_exist(ROOT, recursive = true), "make %", ROOT);
    // Making it again is not an error.
    assert(make_directory_if_it_does_not_exist(ROOT));

    writing_handles();
    appending();
    seeking();
    builders_and_termination();
    empty_and_large();
    names_and_links();
    failures();
    mapping();
    directories();
    hard_links();

    assert(delete_directory(ROOT), "delete %", ROOT);
    assert(!file_exists(ROOT));
    print("ok\n");
}

path :: (name: string) -> string {
    return tprint("%/%", ROOT, name);
}

contents :: (name: string) -> string {
    text, ok := read_entire_file(name);
    assert(ok, "read %", name);
    return text;
}

// A handle opened for writing, with or without the old content, reads too.
writing_handles :: () {
    name := path("fresh.txt");
    assert(write_entire_file(name, "old content that goes away"));
    file, opened := file_open(name, for_writing = true);
    assert(opened && is_valid(file));
    // Truncated on open.
    length, length_ok := file_length(file);
    assert(length_ok && length == 0, "truncated length %", length);
    assert(file_write(*file, "0123456789"));
    position, position_ok := file_current_position(file);
    assert(position_ok && position == 10, "position after write %", position);
    assert(file_set_position(file, 2));
    buffer: [4] u8;
    ok, count := file_read(file, buffer.data, buffer.count);
    assert(ok && count == 4 && cast(string) buffer == "2345", "read back through a fresh writing handle: % % '%'", ok, count, cast(string) buffer);
    // Overwrite in the middle.
    assert(file_set_position(file, 4));
    assert(file_write(*file, "ab"));
    assert(file_close(*file));
    assert(!is_valid(file));
    // Closing an already-closed handle is harmless.
    assert(file_close(*file));
    text := contents(name);
    assert(text == "0123ab6789", "after overwrite '%'", text);

    // A file opened for writing that did not exist is created.
    created := path("created.bin");
    assert(!file_exists(created));
    file, opened = file_open(created, for_writing = true, keep_existing_content = true);
    assert(opened);
    assert(file_write(*file, "abc"));
    assert(file_set_position(file, 0));
    ok, count = file_read(file, buffer.data, 3);
    assert(ok && count == 3 && cast(string) array_view(buffer, 0, 3) == "abc");
    assert(file_close(*file));
    assert(contents(created) == "abc");
}

appending :: () {
    name := path("append.txt");
    assert(write_entire_file(name, "first\n"));
    for 1..3 {
        file, opened := file_open(name, for_writing = true, keep_existing_content = true);
        assert(opened);
        // Opened at the end.
        position := file_current_position(file);
        assert(position == 6 + 7 * (it - 1), "append %: position %", it, position);
        line := tprint("line %\n", it);
        assert(file_write(*file, line));
        assert(file_close(*file));
    }
    text := contents(name);
    assert(text == "first\nline 1\nline 2\nline 3\n", "appended '%'", text);

    // Keeping the content is not append mode: after a seek, writes land where the seek put
    // them, the same on every platform.
    file, opened := file_open(name, for_writing = true, keep_existing_content = true);
    assert(opened);
    assert(file_set_position(file, 0));
    assert(file_write(*file, "F"));
    assert(file_close(*file));
    text = contents(name);
    assert(text == "First\nline 1\nline 2\nline 3\n", "overwritten in place '%'", text);
}

seeking :: () {
    name := path("seek.bin");
    assert(write_entire_file(name, "abcdefghij"));
    file, opened := file_open(name);
    assert(opened);
    defer file_close(*file);
    assert(file_seek(file, -3, .END));
    ok, position := file_tell(file);
    assert(ok && position == 7, "seek from end %", position);
    assert(file_seek(file, -2, .CURRENT));
    position = file_current_position(file);
    assert(position == 5, "seek back %", position);
    byte: u8;
    read_ok, count := file_read(file, *byte, 1);
    assert(read_ok && count == 1 && byte == #char "f");

    // Past the end: the seek works, the read finds nothing and does not fail.
    assert(file_set_position(file, 100));
    buffer: [8] u8;
    read_ok, count = file_read(file, buffer.data, buffer.count);
    assert(read_ok && count == 0, "read past the end: % %", read_ok, count);
    // A short read at the tail succeeds with what is there.
    assert(file_set_position(file, 7));
    read_ok, count = file_read(file, buffer.data, buffer.count);
    assert(read_ok && count == 3 && cast(string) array_view(buffer, 0, 3) == "hij", "tail % %", read_ok, count);
    // Measuring keeps the position.
    length, length_ok := file_length(file);
    assert(length_ok && length == 10);
    assert(file_current_position(file) == 10);
    // A negative absolute position is refused.
    assert(!file_set_position(file, -1));

    // Writing past the end fills the gap with zeros.
    sparse := path("sparse.bin");
    output, created := file_open(sparse, for_writing = true);
    assert(created);
    assert(file_write(*output, "A"));
    assert(file_set_position(output, 5));
    assert(file_write(*output, "B"));
    assert(file_close(*output));
    text := contents(sparse);
    assert(text.count == 6 && text[0] == #char "A" && text[5] == #char "B", "sparse count %", text.count);
    for 1..4 assert(text[it] == 0, "gap byte % is %", it, text[it]);
}

builders_and_termination :: () {
    name := path("builder.txt");
    builder: String_Builder;
    for 1..2000 print_to_builder(*builder, "%,", it);
    expected := builder_to_string(*builder, do_reset = false);
    assert(write_entire_file(name, *builder));
    // The builder was reset.
    assert(builder_string_length(*builder) == 0);
    assert(contents(name) == expected);

    // Keeping the builder's contents, and writing it through an open handle.
    append(*builder, "kept");
    file, opened := file_open(name, for_writing = true);
    assert(opened);
    assert(file_write(*file, *builder, do_reset = false));
    assert(builder_string_length(*builder) == 4);
    assert(file_write(*file, *builder));
    assert(builder_string_length(*builder) == 0);
    assert(file_close(*file));
    assert(contents(name) == "keptkept");
    assert(write_entire_file(name, *builder, do_reset = false));
    assert(contents(name) == "");

    // Raw pointer writes, and a negative count is refused.
    bytes := u8.[1, 2, 0, 255];
    assert(write_entire_file(name, bytes.data, bytes.count));
    raw := contents(name);
    assert(raw.count == 4 && raw[3] == 255 && raw[2] == 0);
    assert(!write_entire_file(name, bytes.data, -1));

    terminated, ok := read_entire_file(name, zero_terminated = true);
    assert(ok && terminated.count == 4 && terminated.data[4] == 0);
    // read_entire_file_terminated and the File overload agree.
    other, other_ok := read_entire_file_terminated(name, true);
    assert(other_ok && other == terminated);
    file, opened = file_open(name);
    assert(opened);
    assert(file_set_position(file, 3));
    whole, whole_ok := read_entire_file(file);
    assert(whole_ok && whole == raw, "the File overload reads from the start");
    assert(file_close(*file));
}

empty_and_large :: () {
    empty := path("empty");
    assert(write_entire_file(empty, ""));
    assert(file_exists(empty));
    text, ok := read_entire_file(empty);
    assert(ok && text.count == 0);
    text, ok = read_entire_file(empty, zero_terminated = true);
    assert(ok && text.count == 0);
    file, opened := file_open(empty);
    assert(opened);
    length := file_length(file);
    assert(length == 0);
    byte: u8;
    read_ok, count := file_read(file, *byte, 1);
    assert(read_ok && count == 0);
    assert(file_close(*file));

    // Several megabytes with a pattern that catches dropped or reordered chunks.
    SIZE :: 5 * 1024 * 1024 + 123;
    data := alloc_string(SIZE);
    for i: 0..SIZE - 1 data[i] = cast,trunc(u8) (i * 7 + i / 4096);
    large := path("large.bin");
    assert(write_entire_file(large, data));
    back, back_ok := read_entire_file(large);
    assert(back_ok && back.count == SIZE, "large count %", back.count);
    assert(back == data, "large contents differ");
    // Read in odd-sized pieces through a handle.
    file, opened = file_open(large);
    assert(opened);
    file_size := file_length(file);
    assert(file_size == SIZE);
    piece: [1000] u8;
    offset := 0;
    while true {
        piece_ok, got := file_read(file, piece.data, piece.count);
        assert(piece_ok);
        if got == 0 break;
        assert(memcmp(piece.data, data.data + offset, got) == 0, "piece at %", offset);
        offset += got;
    }
    assert(offset == SIZE, "read % in pieces", offset);
    assert(file_close(*file));
}

names_and_links :: () {
    // Non-ASCII names and contents.
    name := path("naïve-日本語-ü.txt");
    payload := "grüße, 世界 ☃\n";
    assert(write_entire_file(name, payload));
    assert(file_exists(name));
    assert(contents(name) == payload);
    moved := path("moved-ß.txt");
    assert(file_move(name, moved));
    assert(!file_exists(name) && file_exists(moved));
    // Moving over an existing file replaces it.
    other := path("other.txt");
    assert(write_entire_file(other, "other"));
    assert(file_move(other, moved));
    assert(!file_exists(other));
    assert(contents(moved) == "other");

    assert(file_delete(moved));
    assert(!file_exists(moved));

    absolute, absolute_ok := get_absolute_path(ROOT);
    assert(absolute_ok, "absolute path of %", ROOT);
    assert(absolute.count > ROOT.count && contains(absolute, "file-handles"), "absolute '%'", absolute);
    #if OS == .WINDOWS {
        assert(absolute.count > 2 && absolute[1] == #char ":", "drive in '%'", absolute);
        // Known folders.
        program_data := get_path_os_program_data();
        saved_games := get_path_os_saved_games();
        assert(program_data.count > 3 && program_data[1] == #char ":" && file_exists(program_data), "program data '%'", program_data);
        // A service account may have no Saved Games folder; when there is one it is absolute.
        assert(saved_games.count == 0 || saved_games.count > 3 && saved_games[1] == #char ":", "saved games '%'", saved_games);
    } else {
        assert(absolute[0] == #char "/", "rooted '%'", absolute);
    }
}

failures :: () {
    missing := path("no/such/file.txt");
    file, opened := file_open(missing, log_errors = false);
    assert(!opened && !is_valid(file));
    text, ok := read_entire_file(missing, log_errors = false);
    assert(!ok && text.count == 0);
    // A file in a missing directory cannot be created.
    assert(!write_entire_file(missing, "x"));
    assert(!file_exists(missing));
    assert(!file_delete(missing));
    assert(!file_move(missing, path("anything")));
    assert(!delete_directory(path("no-such-directory")));

    // Operations on an invalid handle fail rather than crash.
    nothing: File;
    assert(!is_valid(nothing));
    byte: u8;
    read_ok := file_read(nothing, *byte, 1);
    assert(!read_ok);
    assert(!file_write(*nothing, "x"));
    assert(!file_seek(nothing, 0, .START));
    _, length_ok := file_length(nothing);
    assert(!length_ok);
    tell_ok := file_tell(nothing);
    assert(!tell_ok);
    assert(file_close(*nothing));

    // Bad counts and null buffers.
    name := path("args.txt");
    assert(write_entire_file(name, "abc"));
    file, opened = file_open(name);
    assert(opened);
    count: s64;
    read_ok, count = file_read(file, *byte, 0);
    assert(!read_ok && count == 0);
    read_ok, count = file_read(file, null, 1);
    assert(!read_ok && count == 0);
    read_ok, count = file_read_checked(file, *byte, -1);
    assert(!read_ok);
    // Writing through a read-only handle fails.
    assert(!file_write(*file, "zz"));
    assert(file_close(*file));
    assert(contents(name) == "abc");
    // A directory is not a file to read.
    _, dir_ok := read_entire_file(ROOT, log_errors = false);
    assert(!dir_ok, "reading a directory as a file");
}

mapping :: () {
    name := path("mapped.txt");
    assert(write_entire_file(name, "mapped bytes"));
    info, ok := map_entire_file_start(name);
    assert(ok && info.data == "mapped bytes", "mapped '%'", info.data);
    map_entire_file_end(*info);
    assert(info.data.count == 0);

    empty := path("mapped-empty.txt");
    assert(write_entire_file(empty, ""));
    info, ok = map_entire_file_start(empty);
    assert(ok && info.data.count == 0);
    map_entire_file_end(*info);

    _, missing_ok := map_entire_file_start(path("not-there"), log_errors = false);
    assert(!missing_ok);

    wrapped := string_to_map_file_info("in memory");
    assert(wrapped.data == "in memory");
}

directories :: () {
    // Recursive creation, with a trailing separator, and Windows separators on Windows.
    nested := path("a/b/c/");
    assert(make_directory_if_it_does_not_exist(nested, recursive = true));
    assert(file_exists(path("a/b/c")));
    #if OS == .WINDOWS {
        assert(make_directory_if_it_does_not_exist(tprint("%\\x\\y", ROOT), recursive = true));
        assert(file_exists(path("x/y")));
    }
    // Without recursion a missing parent fails.
    assert(!make_directory_if_it_does_not_exist(path("p/q")));
    assert(!file_exists(path("p")));
    // A file in the way.
    assert(write_entire_file(path("blocker"), "x"));
    assert(!make_directory_if_it_does_not_exist(path("blocker")));
    assert(!make_directory_if_it_does_not_exist(""));

    // Deleting a non-empty tree, with files at several depths.
    assert(write_entire_file(path("a/top.txt"), "1"));
    assert(write_entire_file(path("a/b/mid.txt"), "2"));
    assert(write_entire_file(path("a/b/c/deep.txt"), "3"));
    assert(make_directory_if_it_does_not_exist(path("a/empty")));
    assert(delete_directory(path("a")));
    assert(!file_exists(path("a")));
    assert(!file_exists(path("a/b/c/deep.txt")));
    // Deleting a file with delete_directory fails and leaves it.
    assert(!delete_directory(path("blocker")));
    assert(file_exists(path("blocker")));
}

// Last, so a failure here leaves the rest checked.
hard_links :: () {
    name := path("linked.txt");
    assert(write_entire_file(name, "original"));
    link := path("link.txt");
    assert(file_link(link, name), "file_link");
    assert(contents(link) == "original");
    // A second name for the same file, not a copy: a write through one shows through the other.
    assert(write_entire_file(name, "changed"));
    assert(contents(link) == "changed", "the link sees the write");
    // The name is taken now.
    assert(!file_link(link, name));
    assert(!file_link(path("other-link.txt"), path("missing")));
    assert(file_delete(link));
    assert(file_exists(name));
    assert(file_delete(name));
}
