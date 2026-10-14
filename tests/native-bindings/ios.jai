I :: #import "iOS";

libc :: #library,system "c";
abort :: () #foreign libc;

check :: (value: bool) {
    if !value abort();
}

main :: () {
    check(I.get_ios_target_triple(.ARM64) == "arm64-apple-ios");
    check(I.get_ios_target_triple(.X64) == "x86_64-apple-ios");
    a := I.get_ios_sdk_path();
    b := I.get_ios_sdk_path();

    check(a == b);
    c := I.get_ios_simulator_sdk_path();
    d := I.get_ios_simulator_sdk_path();

    check(c == d);
}
