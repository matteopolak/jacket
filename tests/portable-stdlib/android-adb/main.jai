// Reviewed Android/Toolchain API and official SDK CLI protocol.
// This native protocol fixture is not an Android SDK/device integration claim.
#import "Basic";
A :: #import "Android/Toolchain";
S :: #import "String";

exercise :: () {
    path := sprint("%/../../../target/portable-adb/sdk protocol fixture", S.path_strip_filename(#file));

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
    free(info.protocol_version);
    free(info.tool_version);
    free(info.exe_path);
    devices_ok, devices := A.adb_devices();

    assert(devices_ok && devices.count == 3);
    assert(devices[0].serial == "ready-1" && devices[0].connection_state == .READY);
    assert(devices[0].model_name == "Pixel_8" && devices[0].transport_id == 12);
    assert(devices[1].connection_state == .UNAUTHORIZED && devices[2].connection_state == .OFFLINE);

    for device: devices {
        free(device.serial);
        free(device.product_name);
        free(device.model_name);
        free(device.device_name);
    }

    free(devices.data);
    devices_ok, devices = A.adb_devices(return_only_ready_devices = true);
    assert(devices_ok && devices.count == 1 && devices[0].serial == "ready-1");

    for device: devices {
        free(device.serial);
        free(device.product_name);
        free(device.model_name);
        free(device.device_name);
    }

    free(devices.data);
    installed, sdk_path := A.is_adb_installed();

    assert(installed && sdk_path == "/explicit/sdk/platform-tools/adb");
    free(sdk_path);
    malformed := sprint("%/../../../target/portable-adb/malformed SDK fixture", S.path_strip_filename(#file));

    defer free(malformed);
    A.__adb_path = malformed;
    info, parsed = A.adb_version();
    assert(!parsed && info.protocol_version == "" && info.tool_version == "" && info.exe_path == "");
    devices_ok, devices = A.adb_devices();
    assert(!devices_ok && devices.count == 0);
    A.__adb_path = path;
    A.adb_device_serial = "selected serial";
    ok, output, errors, code := A.adb("echo-args", "a b", "'quoted'", device_serial = "explicit");

    assert(ok && code == 0 && errors == "" && output == "-s\nexplicit\necho-args\na b\n'quoted'\n");
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
    running, queried := A.is_app_running("org.example.app");

    assert(running && queried);
    running, queried = A.is_app_running("org.example.absent");
    assert(!running && queried);
    running, queried = A.is_app_running("org.example.offline");
    assert(!running && !queried);
    running, queried = A.is_app_running("org.example.badpid");
    assert(!running && !queried);
    pid_ok, pid = A.adb_shell_pidof("org.example.badpid");
    assert(!pid_ok && pid == 0);

    for name: string.["org.example.overflow", "org.example.zero", "org.example.badsecond"] {
        running, queried = A.is_app_running(name);
        assert(!running && !queried);
        pid_ok, pid = A.adb_shell_pidof(name);
        assert(!pid_ok && pid == 0);
    }

    assert(A.android_is_app_installed("org.example.app"));
    assert(!A.android_is_app_installed("org.example.absent"));
    assert(!A.android_is_app_installed("org.example.offline"));
    assert(!A.android_is_app_installed("org.example.badpath"));
    assert(A.stop_app("org.example.app"));
    assert(!A.stop_app("org.example.offline"));
    assert(A.adb_uninstall("org.example.app"));
    A.__adb_path = "/definitely/unavailable/adb";
    ok, output, errors, code = A.adb("version");
    assert(!ok && code != 0);
    free(output);
    free(errors);
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
    push_context scoped exercise();
    assert(live == 0);
    print("ok\n");
}
