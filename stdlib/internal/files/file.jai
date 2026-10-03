#if OS == .WINDOWS #load "native-windows.jai";
else #load "native-unix.jai";

USE_MAP_FILE :: OS == .WINDOWS;

STORE_MAPPED_FILENAMES :: false;

DUMB_LINUX_FAILURE_NUMBER :: 0x7fffffffffffffff;

Map_File_Info :: struct {
    data: string;
    #if USE_MAP_FILE map_info: File_Map_Info;
}

file_tell :: (file: File) -> (success: bool, position: s64) {
    position, ok := file_current_position(file);
    return ok, position;
}

file_write :: inline (file: *File, data: string) -> bool {
    return file_write(file, data.data, data.count);
}

file_write :: (file: *File, builder: *String_Builder, do_reset := true) -> bool {
    defer {
        if do_reset reset(builder);
    }

    if builder.failed return false;
    if !builder.initialized return true;

    buffer := get_base_buffer(builder);
    while buffer {
        if !file_write(file, get_buffer_data(buffer), buffer.count) return false;
        buffer = buffer.next;
    }

    return true;
}

read_entire_file :: (f: File, zero_terminated := false) -> string, bool {
    position, ok := file_current_position(f);
    if !ok return "", false;

    length, valid := file_length(f);
    if !valid || length < position return "", false;

    count := length - position;
    allocated := count;

    if zero_terminated {
        if allocated == 0x7fffffffffffffff return "", false;
        allocated += 1;
    }

    memory := cast(*u8) alloc(allocated);
    if !memory && allocated > 0 return "", false;

    result := string.{ count, memory };

    if count > 0 {
        read_ok, total := file_read(f, result.data, count);
        if !read_ok {
            free(result);
            return "", false;
        }

        result.count = total;
    }

    if zero_terminated result.data[result.count] = 0;

    return result, true;
}

read_entire_file :: (name: string, zero_terminated := false, log_errors := true) -> string, bool {
    file, ok := file_open(name, log_errors = log_errors);
    if !ok return "", false;
    defer file_close(*file);
    return read_entire_file(file, zero_terminated);
}

write_entire_file :: inline (name: string, data: string) -> bool {
    return write_entire_file(name, data.data, data.count);
}

write_entire_file :: (name: string, data: *void, count: int) -> bool {
    file, ok := file_open(name, true);
    if !ok return false;
    defer file_close(*file);
    return file_write(*file, data, count) && file_flush(file);
}

write_entire_file :: (name: string, builder: *String_Builder, do_reset := true) -> bool {
    file, ok := file_open(name, true);
    if !ok {
        if do_reset reset(builder);
        return false;
    }

    defer file_close(*file);

    return file_write(*file, builder, do_reset) && file_flush(file);
}

string_to_map_file_info :: (data: string) -> Map_File_Info {
    return .{ data = data };
}

map_entire_file_start :: (name: string, log_errors := true) -> Map_File_Info, bool {
    #if USE_MAP_FILE {
        mapping, ok := map_file(name, log_errors);
        return .{ data = mapping.data, map_info = mapping }, ok;
    } else {
        data, ok := read_entire_file(name, log_errors = log_errors);
        return .{ data = data }, ok;
    }
}

map_entire_file_end :: (file_info: *Map_File_Info) {
    #if USE_MAP_FILE {
        if !munmap_file(*file_info.map_info) {
            context.logger("map_entire_file_end failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
            return;
        }
    } else free(file_info.data);

    file_info.data = "";
}

#scope_module
Seek_Type :: enum {
    START :: 0;
    CURRENT :: 1;
    END :: 2;
}

#import "Basic";
Strings :: #import "String";
