Processor_Query_Type :: enum u8 {
    ALL_PHYSICAL :: 0;
    ALL_LOGICAL :: 1;
    PERFORMANCE_PHYSICAL :: 2;
    PERFORMANCE_LOGICAL :: 3;
}

#if CPU == .X64 {
    M64 :: #import "Machine_X64";

    read_cpu_counter :: inline () -> u64 {
        return M64.rdtsc();
    }
} else {
    read_cpu_counter :: inline () -> u64 {
        sampled: P.timespec;
        result := P.clock_gettime(.MONOTONIC_RAW, *sampled);

        if result != 0 {
            native_failure("raw monotonic counter query failed\n");
            assert(false, "raw monotonic counter query failed");
            return 0;
        }

        return cast(u64) sampled.tv_nsec;
    }
}

get_username :: () -> string, success: bool {
    success, account := get_passwd_for_user();
    if !success || !account.pw_name return "", false;
    return temporary_copy(to_string(account.pw_name)), true;
}

get_home_directory :: (username := "") -> string, success: bool {
    success, account := get_passwd_for_user(username);
    if !success || !account.pw_dir return "", false;
    return temporary_copy(to_string(account.pw_dir)), true;
}

get_machinename :: () -> string {
    capacity: s64 = 256;

    while true {
        buffer := talloc_string(capacity);

        memset(buffer.data, 0, capacity);
        status := P.gethostname(buffer.data, cast(u64) capacity);
        error := P.errno();

        if status == 0 {
            length: s64;
            while length < capacity && buffer[length] != 0 length += 1;

            if length < capacity {
                buffer.count = length;
                return buffer;
            }
        } else if error != P.ENAMETOOLONG {
            native_failure("machine-name lookup failed\n");
            return "";
        }

        assert(capacity <= 0x3fffffffffffffff);
        capacity *= 2;
    }

    return "";
}

get_path_of_running_executable :: () -> string {
    #if OS == .MACOS {
        required: u32 = 1;
        initial: u8;

        if c_executable_path(*initial, *required) == 0 return temporary_copy(to_string(*initial));

        while true {
            if required == 0 {
                native_failure("executable path has invalid size\n");
                return "";
            }

            buffer := talloc_string(cast(s64) required);
            supplied := required;

            if c_executable_path(buffer.data, *supplied) == 0 {
                buffer.count = c_style_strlen(buffer.data);
                return buffer;
            }

            if supplied <= required {
                native_failure("executable-path lookup failed\n");
                return "";
            }

            required = supplied;
        }
    } else {
        capacity: s64 = 256;
        path := temp_c_string("/proc/self/exe");

        while true {
            buffer := talloc_string(capacity);
            count := P.readlink(path, buffer.data, cast(u64) capacity);

            if count < 0 {
                native_failure("executable-path lookup failed\n");
                return "";
            }

            if count < capacity {
                buffer.count = count;
                return buffer;
            }

            assert(capacity <= 0x3fffffffffffffff);
            capacity *= 2;
        }
    }

    return "";
}

get_number_of_processors :: ($type: Processor_Query_Type = .PERFORMANCE_LOGICAL) -> count: s32 {
    #if OS == .MACOS {
        name := "hw.logicalcpu";

        #if type == .ALL_PHYSICAL || type == .PERFORMANCE_PHYSICAL name = "hw.physicalcpu";

        #if type == .PERFORMANCE_PHYSICAL || type == .PERFORMANCE_LOGICAL {
            performance := "hw.perflevel0.logicalcpu";
            #if type == .PERFORMANCE_PHYSICAL performance = "hw.perflevel0.physicalcpu";
            performance_count, performance_found := query_count(performance);
            if performance_found return performance_count;
        }

        value, found := query_count(name);
        if found return value;

        native_failure("processor-count lookup failed\n");

        return 0;
    } else {
        count := P.sysconf(P._SC_NPROCESSORS_ONLN);
        if count <= 0 || count > 0x7fffffff {
            native_failure("processor-count lookup failed\n");
            return 0;
        }

        return cast(s32) count;
    }
}

get_error_string :: (error_code: P.OS_Error_Code) -> string {
    description := c_strerror(cast(s32) error_code);
    if !description return "";
    return temporary_copy(to_string(description));
}

get_error_value_and_string :: () -> (error_code: P.OS_Error_Code, description: string) {
    code := P.errno();
    return code, get_error_string(code);
}

get_passwd_for_user :: (username := "") -> (success: bool, p: P.passwd) {
    name := temp_c_string(username);
    size := P.sysconf(P._SC_GETPW_R_SIZE_MAX);

    if size < 1 size = 1024;

    while true {
        buffer := cast(*u8) talloc(size);
        account: P.passwd;
        found: *P.passwd;
        error: s32;

        if username.count == 0 error = P.getpwuid_r(P.getuid(), *account, buffer, cast(u64) size, *found);
        else error = P.getpwnam_r(name, *account, buffer, cast(u64) size, *found);
        if error == 0 return found != null, account;

        if error != cast(s32) P.ERANGE {
            P.set_errno(cast(P.OS_Error_Code) error);
            return false, .{};
        }

        assert(size <= 0x3fffffffffffffff);
        size *= 2;
    }

    return false, .{};
}

#scope_file
#import "Basic";
P :: #import "POSIX";

libc :: #system_library "libc";
c_strerror :: (error: s32) -> *u8 #foreign libc "strerror";

native_failure :: (message: string) {
    context.logger(message, context.logger_data, Log_Info.{ common_flags = .ERROR });
}

temporary_copy :: (source: string) -> string {
    result := talloc_string(source.count);
    memcpy(result.data, source.data, source.count);
    return result;
}

#if OS == .MACOS {
    c_executable_path :: (buffer: *u8, size: *u32) -> s32 #foreign libc "_NSGetExecutablePath";
    c_sysctlbyname :: (name: *u8, old: *void, old_size: *u64, new: *void, new_size: u64) -> s32 #foreign libc "sysctlbyname";

    query_count :: (name: string) -> s32, bool {
        value: s32;
        bytes: u64 = 4;
        status := c_sysctlbyname(temp_c_string(name), *value, *bytes, null, 0);
        return value, status == 0 && bytes == 4 && value > 0;
    }
}
