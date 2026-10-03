target_triple :: (cpu: CPU_Tag) -> string {
    if cpu == .ARM64 return "arm64-apple-ios";
    return "x86_64-apple-ios";
}

ios_sdk_path :: () -> string {
    if ios_path.count == 0 ios_path = sdk_lookup("xcrun --sdk iphoneos --show-sdk-path");
    return ios_path;
}

simulator_sdk_path :: () -> string {
    if simulator_path.count == 0 simulator_path = sdk_lookup("xcrun --sdk iphonesimulator --show-sdk-path");
    return simulator_path;
}

#scope_file
Native :: #import "internal/bindings/native";

ios_path: string;
simulator_path: string;
libc :: #library,system "c";
sdk_popen :: (command: *u8, mode: *u8) -> *void #foreign libc "popen";
sdk_pclose :: (stream: *void) -> s32 #foreign libc "pclose";
sdk_fread :: (ptr: *void, size: u64, count: u64, stream: *void) -> u64 #foreign libc "fread";
sdk_ferror :: (stream: *void) -> s32 #foreign libc "ferror";
sdk_write :: (fd: s32, ptr: *u8, count: u64) -> s64 #foreign libc "write";
sdk_abort :: () #foreign libc "abort";

sdk_log_failure :: (message: string) {
    written: s64;
    while written < message.count {
        result := sdk_write(2, message.data + written, cast(u64) (message.count - written));
        if result <= 0 sdk_abort();
        written += result;
    }
}

#scope_module
sdk_lookup :: (command: *u8) -> string {
    stream := sdk_popen(command, "r");
    if !stream {
        message := "iOS SDK lookup: could not invoke xcrun\n";
        sdk_log_failure(message);
        return "";
    }

    capacity: s64 = 256;
    buffer := cast(*u8) Native.allocate(capacity);

    defer Native.release(buffer);
    n: s64;
    failed := false;

    while true {
        if n == capacity {
            if capacity > 0x3fff_ffff_ffff_ffff sdk_abort();

            capacity *= 2;
            grown := cast(*u8) Native.allocate(capacity);
            for i: 0..n - 1 grown[i] = buffer[i];

            Native.release(buffer);
            buffer = grown;
        }

        received := cast(s64) sdk_fread(buffer + n, 1, cast(u64) (capacity - n), stream);

        n += received;

        if sdk_ferror(stream) != 0 {
            failed = true;
            break;
        }

        if received == 0 break;
    }

    status := sdk_pclose(stream);

    if failed || status != 0 || n == 0 {
        message := "iOS SDK lookup: xcrun failed or returned no usable path\n";
        sdk_log_failure(message);
        return "";
    }

    start: s64 = 0;
    while start < n && (buffer[start] == 32 || buffer[start] == 9 || buffer[start] == 10 || buffer[start] == 13) start += 1;
    while n > start && (buffer[n - 1] == 32 || buffer[n - 1] == 9 || buffer[n - 1] == 10 || buffer[n - 1] == 13) n -= 1;

    if n == start {
        sdk_log_failure("iOS SDK lookup: xcrun returned an empty path\n");
        return "";
    }

    data := cast(*u8) Native.allocate(n - start);
    for i: 0..n - start - 1 data[i] = buffer[start + i];

    return string.{ data = data, count = n - start };
}
