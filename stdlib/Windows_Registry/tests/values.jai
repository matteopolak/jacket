// test: only(.WINDOWS) timeout(30)
// Reads values every Windows install has, then creates, reads back, enumerates and deletes
// a key under HKCU\Software\JacketTest.
#import "Basic";
#import "String";
R :: #import "Windows_Registry";
W :: #import "Windows";

advapi :: #library,system "advapi32";
RegCreateKeyExA :: (key: W.HKEY, sub_key: *u8, reserved: u32, class: *u8, options: u32, access: R.REGSAM, security: *void, result: *W.HKEY, disposition: *u32) -> R.LSTATUS #foreign advapi;
RegSetValueExA :: (key: W.HKEY, name: *u8, reserved: u32, kind: u32, data: *void, size: u32) -> R.LSTATUS #foreign advapi;
RegDeleteKeyA :: (key: W.HKEY, sub_key: *u8) -> R.LSTATUS #foreign advapi;

REG_SZ :: 1;
REG_DWORD :: 4;
ERROR_SUCCESS :: 0;
ERROR_FILE_NOT_FOUND :: 2;
ERROR_NO_MORE_ITEMS :: 259;

check :: (condition: bool, what: string) {
    if !condition {
        print("FAIL: %\n", what);
        exit(1);
    }
}

main :: () {
    // Values that exist on every Windows install.
    name: [128] u8;
    size: u32 = 128;
    status := R.RegGetValueA(R.HKEY_LOCAL_MACHINE, "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion", "ProductName", R.RRF_RT_REG_SZ, null, name.data, *size);

    check(status == ERROR_SUCCESS, "read ProductName");
    product := string.{ count = cast(s64) size - 1, data = name.data };

    check(begins_with(product, "Windows"), "ProductName starts with Windows");
    print("product name: Windows\n");

    build: u32;
    build_size: u32 = 4;

    status = R.RegGetValueA(R.HKEY_LOCAL_MACHINE, "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion", "CurrentMajorVersionNumber", R.RRF_RT_REG_DWORD, null, *build, *build_size);
    check(status == ERROR_SUCCESS, "read CurrentMajorVersionNumber");
    print("major version: %\n", build);

    missing: u32;
    missing_size: u32 = 4;

    status = R.RegGetValueA(R.HKEY_CURRENT_USER, "Software\\JacketTest\\DoesNotExist", "x", R.RRF_RT_REG_DWORD, null, *missing, *missing_size);
    check(status == ERROR_FILE_NOT_FOUND, "a missing key reports ERROR_FILE_NOT_FOUND");
    print("missing key: file not found\n");

    // Create, write, read back, enumerate, delete.
    RegDeleteKeyA(R.HKEY_CURRENT_USER, "Software\\JacketTest\\child");
    RegDeleteKeyA(R.HKEY_CURRENT_USER, "Software\\JacketTest");

    key: W.HKEY;

    check(RegCreateKeyExA(R.HKEY_CURRENT_USER, "Software\\JacketTest", 0, null, 0, R.KEY_WRITE | R.KEY_READ, null, *key, null) == ERROR_SUCCESS, "create the test key");
    text := "hello registry\0";

    check(RegSetValueExA(key, "text", 0, REG_SZ, text.data, cast(u32) text.count) == ERROR_SUCCESS, "write a string");
    number: u32 = 0x1badcafe;

    check(RegSetValueExA(key, "number", 0, REG_DWORD, *number, 4) == ERROR_SUCCESS, "write a dword");
    child: W.HKEY;

    check(RegCreateKeyExA(key, "child", 0, null, 0, R.KEY_WRITE, null, *child, null) == ERROR_SUCCESS, "create a child key");
    R.RegCloseKey(child);
    R.RegCloseKey(key);

    read_text: [64] u8;
    read_size: u32 = 64;

    check(R.RegGetValueA(R.HKEY_CURRENT_USER, "Software\\JacketTest", "text", R.RRF_RT_REG_SZ, null, read_text.data, *read_size) == ERROR_SUCCESS, "read the string back");
    print("string: %\n", string.{ count = cast(s64) read_size - 1, data = read_text.data });

    read_number: u32;
    read_number_size: u32 = 4;

    check(R.RegGetValueA(R.HKEY_CURRENT_USER, "Software\\JacketTest", "number", R.RRF_RT_REG_DWORD, null, *read_number, *read_number_size) == ERROR_SUCCESS, "read the dword back");
    print("dword: 0x%\n", formatInt(read_number, base = 16));

    // The wide-character open and query agree with the narrow read.
    wide_key: W.HKEY;
    path := "Software\\JacketTest";
    wide_path: [32] u16;

    for 0..path.count - 1 wide_path[it] = path[it];

    check(R.RegOpenKeyExW(R.HKEY_CURRENT_USER, wide_path.data, 0, R.KEY_READ, *wide_key) == ERROR_SUCCESS, "RegOpenKeyExW");
    wide_name: [7] u16 = .[110, 117, 109, 98, 101, 114, 0];
    wide_value: u32;
    wide_size: u32 = 4;
    wide_type: u32;

    check(R.RegQueryValueExW(wide_key, wide_name.data, null, *wide_type, cast(*u8) *wide_value, *wide_size) == ERROR_SUCCESS, "RegQueryValueExW");
    print("wide dword: 0x% type %\n", formatInt(wide_value, base = 16), wide_type);

    entry: [64] u8;
    entry_size: u32 = 64;

    check(R.RegEnumKeyExA(wide_key, 0, entry.data, *entry_size, null, null, null, null) == ERROR_SUCCESS, "enumerate the child key");
    print("child key: %\n", string.{ count = cast(s64) entry_size, data = entry.data });
    entry_size = 64;
    check(R.RegEnumKeyExA(wide_key, 1, entry.data, *entry_size, null, null, null, null) == ERROR_NO_MORE_ITEMS, "enumeration ends");
    R.RegCloseKey(wide_key);

    check(RegDeleteKeyA(R.HKEY_CURRENT_USER, "Software\\JacketTest\\child") == ERROR_SUCCESS, "delete the child key");
    check(RegDeleteKeyA(R.HKEY_CURRENT_USER, "Software\\JacketTest") == ERROR_SUCCESS, "delete the test key");
    status = R.RegGetValueA(R.HKEY_CURRENT_USER, "Software\\JacketTest", "text", R.RRF_RT_REG_SZ, null, read_text.data, *read_size);
    check(status == ERROR_FILE_NOT_FOUND, "the deleted key is gone");
    print("deleted\n");
}
