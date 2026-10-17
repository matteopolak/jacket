// Reviewed Android/Toolchain API and official SDK CLI protocol.
// This native protocol fixture is not an Android SDK/device integration claim.
#import "Basic";
A :: #import "Android/Toolchain";
S :: #import "String";
F :: #import "File";

warnings: s64;
errors_seen: s64;
assertions_seen: s64;

capture :: (message: string, data: *void, info: Log_Info) {
    if (info.common_flags & .WARNING) != 0 warnings += 1;
    if (info.common_flags & .ERROR) != 0 errors_seen += 1;
}

continue_failure :: (location: Source_Code_Location, message: string) -> bool {
    assertions_seen += 1;
    return false;
}

exercise :: () {
    path := sprint("%/../../../../target/portable-adb/sdk protocol fixture", S.path_strip_filename(#file));

    defer free(path);
    A.__adb_path = path;
    assert(type_info(A.Adb_Device).name == "Adb_Device");
    assert(type_info(A.Adb_Version_Info).name == "Adb_Version_Info");
    #assert A.Adb_Device.Connection_State.READY == .DEVICE;
    #assert !#exists(A.take_word) && !#exists(A.quote_remote);
    #assert !#exists(A.array_add) && !#exists(A.copy_string) && !#exists(A.C);
    info, parsed := A.adb_version();

    assert(parsed && info.protocol_version == "1.0.41" && info.tool_version == "35.0.2-test");
    assert(info.exe_path == "/explicit/sdk/platform-tools/adb");
    devices_ok, devices := A.adb_devices();

    assert(devices_ok && devices.count == 3);
    assert(devices[0].serial == "ready-1" && devices[0].connection_state == .READY);
    assert(devices[0].model_name == "Pixel_8" && devices[0].transport_id == 12);
    assert(devices[1].connection_state == .UNAUTHORIZED && devices[2].connection_state == .OFFLINE);

    devices_ok, devices = A.adb_devices(return_only_ready_devices = true);
    assert(devices_ok && devices.count == 1 && devices[0].serial == "ready-1");

    installed, sdk_path := A.is_adb_installed();
    expected_sdk_path := sprint("%/../../../../target/portable-adb/SDK space/platform-tools/adb", S.path_strip_filename(#file));

    defer free(expected_sdk_path);

    absolute_expected, resolved := F.get_absolute_path(expected_sdk_path);

    defer free(absolute_expected);

    assert(installed && resolved && sdk_path == absolute_expected);
    malformed := sprint("%/../../../../target/portable-adb/malformed SDK fixture", S.path_strip_filename(#file));

    defer free(malformed);
    A.__adb_path = malformed;
    info, parsed = A.adb_version();
    assert(!parsed && info.protocol_version == "" && info.tool_version == "" && info.exe_path == "");
    original_assertion := context.assertion_failed;

    context.assertion_failed = continue_failure;
    devices_ok, devices = A.adb_devices();
    context.assertion_failed = original_assertion;
    assert(!devices_ok && devices.count == 0);
    assert(assertions_seen == 1 && errors_seen == 1);
    A.__adb_path = path;
    A.adb_device_serial = "selected serial";
    ok, output, errors, code := A.adb("echo-args", "a b", "'quoted'", device_serial = "explicit");

    assert(ok && code == 0 && errors == "" && output == "-s\nexplicit\necho-args\na b\n'quoted'\n");
    free(output);
    free(errors);
    ok, output, errors, code = A.adb("shell", "echo-args", "a b", "'a b'", "tab\tvalue", "$value;literal");
    assert(ok && output == "-s\nselected serial\nshell\necho-args\n\"a b\"\n\"'a b'\"\ntab\tvalue\n$value;literal\n");
    free(output);
    free(errors);
    ok, output, errors, code = A.adb("fail");
    assert(!ok && code == 7 && output == "partial\n" && errors == "SDK failure\n");
    free(output);
    free(errors);
    assert(A.adb_push("local space", "/remote space", only_update_newer_files = true, dry_run = true, compression_algorithm = .ZSTD));
    assert(A.adb_pull("/remote space", "local space", disable_compression = true));
    assert(A.adb_shell_mkdir("/remote/a b'c"));
    pid_ok, pid := A.adb_shell_pidof("org.example.app");

    assert(pid_ok && pid == 1234);
    assert(warnings == 1);
    pid_ok, pid = A.adb_shell_pidof("org.example.absent");
    assert(!pid_ok && pid == -1);
    running, queried := A.is_app_running("org.example.app");

    assert(running && queried);
    running, queried = A.is_app_running("org.example.absent");
    assert(!running && queried);
    running, queried = A.is_app_running("org.example.offline");
    assert(!running && !queried);
    running, queried = A.is_app_running("org.example.badpid");
    assert(!running && !queried);
    pid_ok, pid = A.adb_shell_pidof("org.example.badpid");
    assert(!pid_ok && pid == -1);

    for name: string.["org.example.overflow", "org.example.zero", "org.example.badsecond"] {
        running, queried = A.is_app_running(name);
        assert(!running && !queried);
        pid_ok, pid = A.adb_shell_pidof(name);
        if name == "org.example.badsecond" assert(pid_ok && pid == 1234);
        else assert(!pid_ok && pid == -1);
    }

    assert(A.android_is_app_installed("org.example.app"));
    assert(!A.android_is_app_installed("org.example.absent"));
    assert(!A.android_is_app_installed("org.example.offline"));
    assert(!A.android_is_app_installed("org.example.badpath"));
    assert(A.stop_app("org.example.app"));
    assert(!A.stop_app("org.example.offline"));
    assert(A.adb_uninstall("org.example.app"));
    A.__adb_path = "/definitely/unavailable/adb";
    context.assertion_failed = continue_failure;
    ok, output, errors, code = A.adb("version");
    context.assertion_failed = original_assertion;
    assert(!ok && code == 0 && output == "" && errors == "");
    free(output);
    free(errors);
    ok, output, errors, code = A.adb();
    assert(!ok && code == 0 && output == "" && errors == "");
    context.assertion_failed = continue_failure;
    pushed := A.adb_push("a", "b", compression_algorithm = .ZSTD, disable_compression = true);
    pulled := A.adb_pull("a", compression_algorithm = .ANY, disable_compression = true);

    context.assertion_failed = original_assertion;
    assert(!pushed && !pulled && assertions_seen == 4 && errors_seen == 8 && warnings == 2);
    A.__adb_path = "";
    A.adb_device_serial = "";
}

backing: Allocator;
pointers: [4096] *void;
live: s64;

track :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    if mode == .FREE {
        if old_memory {
            found := false;

            for *pointer: pointers if pointer.* == old_memory {
                pointer.* = null;
                live -= 1;
                found = true;
                break;
            }

            assert(found);
        }

        return backing.proc(mode, size, old_size, old_memory, backing.data);
    }

    result := backing.proc(mode, size, old_size, old_memory, backing.data);
    if (mode == .ALLOCATE || mode == .RESIZE) && result {
        if old_memory {
            for *pointer: pointers if pointer.* == old_memory {
                pointer.* = result;
                return result;
            }

            assert(false, "resize ownership missing");
        }

        for *pointer: pointers if pointer.* == null {
            pointer.* = result;
            live += 1;
            return result;
        }

        assert(false, "tracking capacity exhausted");
    }

    return result;
}

main :: () {
    backing = context.allocator;
    scoped := context;
    scoped.allocator = .{ track, null };
    storage: Temporary_Storage_Paged_Alloc;
    memory: [65536] u8;

    storage.data = memory.data;
    storage.size = memory.count;
    scoped.temporary_storage = *storage.base;
    scoped.logger = capture;
    push_context scoped exercise();
    reset_temporary_storage(*storage.base);
    assert(live == 0);
    print("ok\n");
}
