// Private host unit for toolchain operations; not an iOS public-facade runtime pass.
#load "../../stdlib/internal/bindings/apple-toolchain/module.jai";

lib_test :: #library,system "c";
abort :: () #foreign lib_test;
release_test_result :: (memory: *void) #foreign lib_test "free";
report_path :: (format: *u8, args: ..Any) -> s32 #foreign lib_test "printf";

check :: (value: bool) {
    if !value abort();
}

main :: () {
    check(target_triple(.ARM64) == "arm64-apple-ios");
    check(target_triple(.X64) == "x86_64-apple-ios");
    check(target_triple(cast(CPU_Tag) 0xffff_ffff) == "x86_64-apple-ios");
    // Genuine native pipe behavior, not a substitute SDK or public iOS pass.
    long_result := sdk_lookup("i=0; while [ $i -lt 5000 ]; do printf x; i=$((i+1)); done; printf '\\n'");

    check(long_result.count == 5000);
    for byte: long_result check(byte == #char "x");

    release_test_result(long_result.data);
    check(sdk_lookup("printf '  \\n'") == "");
    check(sdk_lookup("exit 7") == "");
    device := ios_sdk_path();
    again := ios_sdk_path();

    check(device == again);
    if device.count > 0 check(device.data == again.data);

    check(device.count <= 0x7fff_ffff);
    check(report_path("SDK_DEVICE_PATH=%.*s\n", cast(s32) device.count, device.data) >= 0);

    simulator := simulator_sdk_path();
    repeated := simulator_sdk_path();

    check(simulator == repeated);
    if simulator.count > 0 check(simulator.data == repeated.data);

    check(simulator.count <= 0x7fff_ffff);
    check(report_path("SDK_SIMULATOR_PATH=%.*s\n", cast(s32) simulator.count, simulator.data) >= 0);
}
