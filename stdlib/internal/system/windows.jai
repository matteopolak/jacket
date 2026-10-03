Processor_Query_Type :: enum u8 {
    ALL_PHYSICAL :: 0;
    ALL_LOGICAL :: 1;
    PERFORMANCE_PHYSICAL :: 2;
    PERFORMANCE_LOGICAL :: 3;
}

get_username :: () -> string, success: bool {
    capacity: u32 = 256;

    while true {
        buffer := cast(*u16) alloc(cast(s64) capacity * 2);

        defer free(buffer);
        count := capacity;
        if W.GetUserNameW(buffer, *count) != 0 return temporary_text(buffer, cast(s64) count - 1);
        if W.GetLastError() != 122 || count <= capacity return "", false;

        capacity = count;
    }

    return "", false;
}

get_home_directory :: (username := "") -> string, success: bool {
    if username.count > 0 return account_profile(username);

    token: W.HANDLE;
    if OpenProcessToken(W.GetCurrentProcess(), 8, *token) == 0 return "", false;

    defer close_native_handle(token);
    capacity: u32;
    queried := GetUserProfileDirectoryW(token, null, *capacity);

    if queried == 0 && W.GetLastError() != 122 return "", false;

    while capacity > 0 {
        buffer := cast(*u16) alloc(cast(s64) capacity * 2);

        defer free(buffer);
        count := capacity;
        if GetUserProfileDirectoryW(token, buffer, *count) != 0 {
            length: s64;
            while length < capacity && buffer[length] != 0 length += 1;
            if length == capacity return "", false;
            return temporary_text(buffer, length);
        }

        if W.GetLastError() != 122 || count <= capacity return "", false;

        capacity = count;
    }

    return "", false;
}

get_machinename :: () -> string {
    capacity: u32 = 256;

    while true {
        buffer := cast(*u16) alloc(cast(s64) capacity * 2);

        defer free(buffer);
        count := capacity;
        if W.GetComputerNameW(buffer, *count) != 0 {
            result, ok := temporary_text(buffer, count);
            assert(ok, "computer name conversion failed");
            return result;
        }

        assert(W.GetLastError() == 111 && count > capacity, "computer name lookup failed");
        capacity = count;
    }

    return "";
}

get_path_of_running_executable :: () -> string {
    capacity: s32 = 256;

    while true {
        buffer := cast(*u16) alloc(cast(s64) capacity * 2);

        defer free(buffer);
        count := W.GetModuleFileNameW(null, buffer, capacity);

        assert(count > 0, "executable path lookup failed");

        if count < capacity {
            result, ok := temporary_text(buffer, count);
            assert(ok, "executable path conversion failed");
            return result;
        }

        assert(capacity <= 0x3fffffff, "executable path exceeds supported size");
        capacity *= 2;
    }

    return "";
}

get_error_string :: (error_code: W.OS_Error_Code) -> string {
    buffer: *u16;
    count := W.FormatMessageW(0x100 | 0x1000 | 0x200, null, error_code, 0, *buffer, 0, null);

    defer if buffer release_local_memory(buffer);
    if count <= 0 return "";

    result, ok := temporary_text(buffer, count);
    if !ok return "";

    return result;
}

get_error_value_and_string :: () -> (error_code: W.OS_Error_Code, description: string) {
    code := W.GetLastError();
    return code, get_error_string(code);
}

read_cpu_counter :: inline () -> u64 {
    #if CPU == .X64 return rdtsc();
    else {
        value: W.LARGE_INTEGER;
        assert(W.QueryPerformanceCounter(*value) != 0, "performance counter query failed");
        return cast(u64) value.QuadPart;
    }
}

get_number_of_processors :: ($type: Processor_Query_Type = .PERFORMANCE_LOGICAL) -> count: s32 {
    bytes: u32;
    queried := W.GetLogicalProcessorInformationEx(.RelationProcessorCore, null, *bytes);

    assert(queried != 0 || W.GetLastError() == 122, "processor topology query failed");

    while bytes > 0 {
        memory := cast(*u8) alloc(bytes);

        defer free(memory);
        supplied := bytes;
        if W.GetLogicalProcessorInformationEx(.RelationProcessorCore, cast(*W.SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX) memory, *supplied) == 0 {
            assert(W.GetLastError() == 122 && supplied > bytes, "processor topology query failed");
            bytes = supplied;
            continue;
        }

        assert(supplied <= bytes, "processor topology exceeds supplied buffer");

        offset: s64;
        best_class: u8;
        total: s64;
        fastest: s64;

        while offset < supplied {
            assert(cast(s64) supplied - offset >= 8, "truncated processor topology header");
            entry := cast(*W.SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX) (memory + offset);
            record_size := cast(s64) entry.Size;

            assert(record_size >= 32 && record_size <= cast(s64) supplied - offset, "invalid processor topology record");
            assert(entry.Relationship == .RelationProcessorCore, "unexpected processor relationship");
            groups := cast(s64) entry.Processor.GroupCount;

            assert(groups > 0 && groups <= (record_size - 32) / size_of(W.GROUP_AFFINITY), "invalid processor affinity count");
            contribution: s64 = 1;

            #if type == .ALL_LOGICAL || type == .PERFORMANCE_LOGICAL {
                contribution = 0;
                masks := cast(*W.GROUP_AFFINITY) *entry.Processor.GroupMask;

                for i: 0..groups - 1 {
                    bits := masks[i].Mask;
                    while bits != 0 {
                        contribution += 1;
                        bits &= bits - 1;
                    }
                }
            }

            total += contribution;
            efficiency := entry.Processor.EfficiencyClass;
            if efficiency > best_class {
                best_class = efficiency;
                fastest = 0;
            }

            if efficiency == best_class fastest += contribution;

            offset += record_size;
        }

        #if type == .PERFORMANCE_LOGICAL || type == .PERFORMANCE_PHYSICAL total = fastest;
        assert(total > 0 && total <= 0x7fffffff, "invalid processor count");

        return cast(s32) total;
    }

    assert(false, "empty processor topology");

    return 0;
}

#scope_file
#import "Basic";
W :: #import "Windows";
Text :: #import "internal/platform-io/windows-text";

advapi32 :: #system_library "advapi32";
userenv :: #system_library "userenv";
kernel32 :: #system_library "kernel32";
#if CPU == .X64 rdtsc :: () -> u64 #intrinsic;

OpenProcessToken :: (process: W.HANDLE, access: u32, token: *W.HANDLE) -> s32 #foreign advapi32;
GetUserProfileDirectoryW :: (token: W.HANDLE, path: *u16, count: *u32) -> s32 #foreign userenv;
LookupAccountNameW :: (system: *u16, account: *u16, sid: *void, sid_bytes: *u32, domain: *u16, domain_chars: *u32, use: *s32) -> s32 #foreign advapi32;
ConvertSidToStringSidW :: (sid: *void, text: **u16) -> s32 #foreign advapi32;
RegOpenKeyExW :: (key: *void, name: *u16, options: u32, access: u32, result: **void) -> s32 #foreign advapi32;
RegQueryValueExW :: (key: *void, value: *u16, reserved: *u32, type: *u32, data: *u8, bytes: *u32) -> s32 #foreign advapi32;
RegCloseKey :: (key: *void) -> s32 #foreign advapi32;
ExpandEnvironmentStringsW :: (source: *u16, destination: *u16, capacity: u32) -> u32 #foreign kernel32;

temporary_text :: (text: *u16, count: s64) -> string, bool {
    result, ok := Text.utf8_text(text, count);
    if !ok return "", false;
    defer free(result);
    return copy_temporary_string(result), true;
}

close_native_handle :: (handle: W.HANDLE) {
    assert(W.CloseHandle(handle) != 0, "native handle cleanup failed");
}

release_local_memory :: (memory: *void) {
    assert(W.LocalFree(memory) == null, "native memory cleanup failed");
}

account_profile :: (username: string) -> string, bool {
    name := Text.utf16_path(username);

    defer free(name);
    if !name return "", false;

    sid_size: u32;
    domain_size: u32;
    use: s32;
    queried := LookupAccountNameW(null, name, null, *sid_size, null, *domain_size, *use);

    if queried == 0 && W.GetLastError() != 122 return "", false;

    sid := alloc(sid_size);

    defer free(sid);
    domain := cast(*u16) alloc((cast(s64) domain_size + 1) * 2);

    defer free(domain);
    if LookupAccountNameW(null, name, sid, *sid_size, domain, *domain_size, *use) == 0 return "", false;

    sid_text: *u16;
    if ConvertSidToStringSidW(sid, *sid_text) == 0 return "", false;

    defer release_local_memory(sid_text);
    sid_length: s64;
    while sid_text[sid_length] != 0 sid_length += 1;

    sid_utf8, encoded := Text.utf8_text(sid_text, sid_length);
    if !encoded return "", false;

    defer free(sid_utf8);
    key_name := Text.utf16_path(tprint("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList\\%", sid_utf8));

    defer free(key_name);
    key: *void;
    if RegOpenKeyExW(cast(*void) cast(s64) -2147483646, key_name, 0, 0x20019, *key) != 0 return "", false;

    defer assert(RegCloseKey(key) == 0, "profile registry cleanup failed");
    value_name := Text.utf16_path("ProfileImagePath");

    defer free(value_name);
    bytes: u32;
    kind: u32;

    if RegQueryValueExW(key, value_name, null, *kind, null, *bytes) != 0 return "", false;
    if bytes % 2 != 0 || (kind != 1 && kind != 2) return "", false;

    value := cast(*u16) alloc(cast(s64) bytes + 2);

    defer free(value);
    capacity := bytes;
    if RegQueryValueExW(key, value_name, null, *kind, cast(*u8) value, *bytes) != 0 || bytes > capacity return "", false;

    value[bytes / 2] = 0;

    if kind == 1 {
        count: s64;
        while value[count] != 0 count += 1;
        return temporary_text(value, count);
    }

    if kind != 2 return "", false;

    needed := ExpandEnvironmentStringsW(value, null, 0);
    while needed > 0 {
        expanded := cast(*u16) alloc(cast(s64) needed * 2);

        defer free(expanded);
        count := ExpandEnvironmentStringsW(value, expanded, needed);
        if count == 0 return "", false;
        if count <= needed return temporary_text(expanded, cast(s64) count - 1);

        needed = count;
    }

    return "", false;
}
