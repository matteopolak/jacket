#import "Basic";
S :: #import "String";
B :: #import "Bindings_Generator";

main :: () {
    for os: Operating_System_Tag.[.LINUX, .ANDROID] {
        for cpu: CPU_Tag.[.X64, .ARM64] {
            generic, platform, ok := B.get_libc_paths(os, cpu);

            assert(ok && S.contains(generic, "/usr/include"));
            assert(platform.count > generic.count);
            assert(S.contains(platform, ifx cpu == .X64 then "x86_64" else "aarch64"));
            free(generic);
            free(platform);
        }
    }

    apple, apple_platform, apple_ok := B.get_libc_paths(.IOS, .ARM64);

    assert(apple_ok && apple == apple_platform && apple.data != apple_platform.data);
    free(apple);
    free(apple_platform);
    invalid, invalid_platform, invalid_ok := B.get_libc_paths(.LINUX, .UNINITIALIZED);

    assert(!invalid_ok && !invalid && !invalid_platform);
    print("ok\n");
}
