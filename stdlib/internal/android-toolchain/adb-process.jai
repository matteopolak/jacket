adb :: (command_args: ..string, device_serial := "", log_adb_command_errors := false, print_captured_output := false, loc := #caller_location, try_and_escape_spaces_in_shell_arguments := true) -> (bool, output_string := "", error_string := "", exit_code := 0) {
    C :: #import "Process";

    if command_args.count == 0 {
        context.logger("ADB requires a command\n", context.logger_data, Log_Info.{ common_flags = .ERROR, location = loc });
        return false, "", "", 0;
    }

    if __adb_path.count == 0 {
        found, path := is_adb_installed();
        if !found {
            context.logger("Android SDK ADB executable was not found\n", context.logger_data, Log_Info.{ common_flags = .ERROR, location = loc });
            return false, "", "", 0;
        }

        if !retain_adb_path(path) return false, "", "", 0;
    }

    // Startup is an essential operation, never an optional assertion argument.
    startup, startup_output, startup_errors, startup_timeout := C.run_command(__adb_path, "start-server", capture_and_return_output = true);

    free(startup_output);
    free(startup_errors);

    if startup.type != .EXITED || startup.exit_code != 0 || startup_timeout {
        adb_failure("ADB server startup failed");
        return false, "", "", 0;
    }

    arguments: [..] string;
    escaped: [..] string;

    defer array_free(arguments);

    defer {
        for value: escaped free(value);
        array_free(escaped);
    }

    array_add(*arguments, __adb_path);
    serial := ifx device_serial.count > 0 then device_serial else adb_device_serial;
    if serial.count > 0 {
        array_add(*arguments, "-s");
        array_add(*arguments, serial);
    }

    shell := command_args.count > 0 && command_args[0] == "shell";

    for argument, index: command_args {
        if shell && index > 0 && try_and_escape_spaces_in_shell_arguments {
            converted := escape_spaces(argument);
            array_add(*escaped, converted);
            array_add(*arguments, converted);
        } else array_add(*arguments, argument);
    }

    result, output, errors, timed_out := C.run_command(..arguments, capture_and_return_output = true,
        print_captured_output = print_captured_output || adb_be_verbose);
    success := result.type == .EXITED && result.exit_code == 0 && !timed_out;
    code := ifx result.type == .EXITED then result.exit_code else 0;

    if !success && log_adb_command_errors {
        context.logger("ADB command failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR, location = loc });
        if errors.count > 0 context.logger(errors, context.logger_data, Log_Info.{ common_flags = .ERROR, location = loc });
    }

    return success, output, errors, code;
}

adb_version :: () -> Adb_Version_Info, bool {
    success, output, errors := adb("version");

    defer free(output);
    defer free(errors);
    info: Adb_Version_Info;
    if !success return info, false;

    remaining := copy_temporary_string(output);
    protocol := take_line(*remaining);
    tool := take_line(*remaining);
    path := take_line(*remaining);

    if remaining.count != 0 || !starts(protocol, "Android Debug Bridge version ") ||
       !starts(tool, "Version ") || !starts(path, "Installed as ") return info, false;

    info.protocol_version = after_prefix(protocol, "Android Debug Bridge version ");
    info.tool_version = after_prefix(tool, "Version ");
    info.exe_path = after_prefix(path, "Installed as ");

    return info, true;
}

adb_devices :: (return_only_ready_devices := false) -> bool, [] Adb_Device {
    success, output, errors := adb("devices", "-l");

    defer free(output);
    defer free(errors);
    devices: [..] Adb_Device;

    defer array_free(devices);

    if !success return false, .{};

    remaining := copy_temporary_string(output);
    if take_line(*remaining) != "List of devices attached" {
        adb_failure("invalid ADB device list heading");
        return false, .{};
    }

    while remaining.count > 0 {
        line := take_line(*remaining);
        if line.count == 0 continue;

        serial := take_word(*line);
        state := take_word(*line);

        if serial.count == 0 || state.count == 0 {
            adb_failure("ADB device row requires serial and state");
            return false, .{};
        }

        device: Adb_Device;
        device.connection_state = parse_state(state);
        if return_only_ready_devices && device.connection_state != .READY continue;

        device.serial = serial;

        while line.count > 0 {
            item := take_word(*line);
            if starts(item, "product:") {
                device.product_name = after_prefix(item, "product:");
            } else if starts(item, "model:") {
                device.model_name = after_prefix(item, "model:");
            } else if starts(item, "device:") {
                device.device_name = after_prefix(item, "device:");
            } else if starts(item, "transport_id:") {
                number, valid, tail := to_integer(after_prefix(item, "transport_id:"));
                if !valid || tail.count != 0 || number < 0 {
                    adb_failure("invalid ADB device transport ID");
                    return false, .{};
                }

                device.transport_id = number;
            }
        }

        array_add(*devices, device);
    }

    if devices.count == 0 return true, .{};

    storage := cast(*Adb_Device) temporary_alloc(devices.count * size_of(Adb_Device));
    if !storage return false, .{};

    memcpy(storage, devices.data, devices.count * size_of(Adb_Device));

    return true, .{ devices.count, storage };
}

#scope_module
#import "Basic";

cached_adb_path: string;
cached_adb_allocator: Allocator;

retain_adb_path :: (path: string) -> bool {
    allocator := context.default_allocator;
    if !allocator.proc {
        adb_failure("ADB cache requires a durable default allocator");
        return false;
    }

    copy: string;

    {
        push_allocator(allocator);
        copy = copy_string(path);
    }

    if copy.count != path.count || (path.count > 0 && !copy.data) return false;

    if cached_adb_path.data {
        push_allocator(cached_adb_allocator);
        free(cached_adb_path);
    }

    cached_adb_path = copy;
    cached_adb_allocator = allocator;
    __adb_path = copy;

    return true;
}

starts :: (text: string, prefix: string) -> bool {
    if text.count < prefix.count return false;
    for index: 0..prefix.count - 1 if text[index] != prefix[index] return false;
    return true;
}

after_prefix :: (text: string, prefix: string) -> string {
    return .{ text.count - prefix.count, text.data + prefix.count };
}

take_line :: (remaining: *string) -> string {
    length: s64;
    while length < remaining.count && remaining.data[length] != 10 length += 1;

    result := string.{ length, remaining.data };
    consumed := min(length + 1, remaining.count);

    remaining.data += consumed;
    remaining.count -= consumed;
    if result.count > 0 && result[result.count - 1] == 13 result.count -= 1;

    return result;
}

take_word :: (remaining: *string) -> string {
    while remaining.count > 0 && (remaining.data[0] == 32 || remaining.data[0] == 9 || remaining.data[0] == 13 || remaining.data[0] == 10) {
        remaining.data += 1;
        remaining.count -= 1;
    }

    length: s64;
    while length < remaining.count && remaining.data[length] != 32 && remaining.data[length] != 9 && remaining.data[length] != 13 && remaining.data[length] != 10 length += 1;

    result := string.{ length, remaining.data };

    remaining.data += length;
    remaining.count -= length;

    return result;
}

parse_state :: (word: string) -> Adb_Device.Connection_State {
    if word == "device" return .DEVICE;
    if word == "offline" return .OFFLINE;
    if word == "unauthorized" return .UNAUTHORIZED;
    if word == "authorizing" return .AUTHORIZING;
    if word == "connecting" return .CONNECTING;
    if word == "no" || word == "noperm" return .NOPERM;
    if word == "bootloader" return .BOOTLOADER;
    if word == "host" return .HOST;
    if word == "recovery" return .RECOVERY;
    if word == "sideload" return .SIDELOAD;

    return .UNKNOWN;
}

adb_failure :: (message: string) {
    context.logger(message, context.logger_data, Log_Info.{ common_flags = .ERROR });
    assert(false, message);
}

escape_spaces :: (text: string) -> string {
    spaces: s64;
    for byte: text if byte == 32 spaces += 1;

    if spaces == 0 return copy_string(text);

    result := alloc_string(text.count + 2);
    index: s64;
    result[index] = 34;
    index += 1;

    for byte: text {
        result[index] = byte;
        index += 1;
    }

    result[index] = 34;

    return result;
}

quote_remote :: (text: string) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, "'");

    for byte: text {
        if byte == 39 append(*builder, "'\"'\"'");
        else append(*builder, *byte, 1);
    }

    append(*builder, "'");

    return builder_to_string(*builder);
}
