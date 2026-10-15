// Reviewed windows-registry.md; official SDK private test setup/cleanup only.
#import "Basic";
R :: #import "Windows_Registry";
W :: #import "Windows";
U :: #import "Windows_Utf8";

main :: () {
    #if OS == .WINDOWS {
        root: W.HKEY;
        status := open_user(R.KEY_READ | R.KEY_WRITE, *root);

        assert(status == 0 && root != null);
        path := sprint("Software\\CleanroomRegistryTest-%", W.GetCurrentProcessId());

        defer free(path);
        wide, wide_valid, wide_bytes := U.utf8_to_wide(path);

        assert(wide_valid && wide != null && wide_bytes > 0);
        defer free(wide);
        key: W.HKEY;
        disposition: u32;

        status = create_key(root, wide, 0, null, 1, R.KEY_READ | R.KEY_WRITE, null, *key, *disposition);
        assert(status == 0 && disposition == 1 && key != null);
        name, name_valid, name_bytes := U.utf8_to_wide("sample");

        assert(name_valid && name != null && name_bytes == 12);
        defer free(name);
        value: u32 = 0x89abcdef;

        status = set_value(key, name, 0, 4, cast(*u8) *value, 4);
        assert(status == 0);

        read: W.HKEY;

        status = R.RegOpenKeyExW(root, wide, 0, R.KEY_READ, *read);
        assert(status == 0 && read != null);
        count: u32;
        kind: u32;

        status = R.RegQueryValueExW(read, name, null, *kind, null, *count);
        assert(status == 0 && kind == 4 && count == 4);
        received: u32;

        count = 1;
        status = R.RegQueryValueExW(read, name, null, *kind, cast(*u8) *received, *count);
        assert(status == 234 && count == 4);
        status = R.RegQueryValueExW(read, name, null, *kind, cast(*u8) *received, *count);
        assert(status == 0 && received == value);
        status = R.RegCloseKey(read);
        assert(status == 0);

        narrow := temp_c_string(path);

        status = R.RegOpenKeyExA(root, narrow, 0, R.KEY_READ, *read);
        assert(status == 0 && read != null);
        count = 4;
        status = R.RegQueryValueExA(read, temp_c_string("sample"), null, *kind, cast(*u8) *received, *count);
        assert(status == 0 && kind == 4 && count == 4 && received == value);
        status = R.RegCloseKey(read);
        assert(status == 0);
        count = 4;
        received = 0;
        status = R.RegGetValueA(root, narrow, temp_c_string("sample"), R.RRF_RT_REG_DWORD, *kind, *received, *count);
        assert(status == 0 && kind == 4 && count == 4 && received == value);
        child: W.HKEY;
        child_name, child_valid, child_bytes := U.utf8_to_wide("child");

        assert(child_valid && child_name != null && child_bytes == 10);
        defer free(child_name);
        status = create_key(key, child_name, 0, null, 1, R.KEY_READ, null, *child, *disposition);
        assert(status == 0 && disposition == 1);
        status = R.RegCloseKey(child);
        assert(status == 0);
        buffer: [32] u8;

        count = buffer.count;
        written: W.FILETIME;

        status = R.RegEnumKeyExA(key, 0, buffer.data, *count, null, null, null, *written);
        assert(status == 0 && count == 5 && to_string(buffer.data) == "child");
        status = delete_tree(key, child_name);
        assert(status == 0);
        status = R.RegCloseKey(key);
        assert(status == 0);
        status = delete_tree(root, wide);
        assert(status == 0);
        status = R.RegOpenKeyExW(root, wide, 0, R.KEY_READ, *read);
        assert(status == 2);
        status = R.RegCloseKey(root);
        assert(status == 0);
        print("ok\n");
    } else assert(false, "Registry execution requires actual Windows");
}

#scope_file
native :: #system_library "advapi32";
open_user :: (access: u32, key: *W.HKEY) -> s32 #foreign native "RegOpenCurrentUser";

create_key :: (parent: W.HKEY, path: *u16, reserved: u32, class: *u16, options: u32,
               access: u32, security: *void, key: *W.HKEY, disposition: *u32) -> s32 #foreign native "RegCreateKeyExW";

set_value :: (key: W.HKEY, name: *u16, reserved: u32, kind: u32, data: *u8, count: u32) -> s32 #foreign native "RegSetValueExW";
delete_tree :: (key: W.HKEY, path: *u16) -> s32 #foreign native "RegDeleteTreeW";
