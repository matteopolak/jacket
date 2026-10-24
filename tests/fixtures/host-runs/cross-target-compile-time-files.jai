// Compile-time code runs on the compiler's host whatever target is selected, so file system
// queries in a plain `#run` see the host's files, not the target's `stat` layout or Win32 calls.
// The directory below is made relative to the program's directory.
#import "Basic";
#import "File";
#import "File_Utilities";
SIMD :: #import "Extensions/SIMD";

#run {
    assert(file_exists(#file), "this file exists");
    assert(!file_exists("/no/such/path/for/this/test"), "a missing path does not exist");

    is_dir, found := is_directory(#filepath);

    assert(found && is_dir, "the fixture directory is a directory");
    is_dir, found = is_directory(#file);
    assert(found && !is_dir, "this file is not a directory");

    assert(make_directory_if_it_does_not_exist("cross-target-files/a/b", true), "recursive make");
    is_dir, found = is_directory("cross-target-files/a/b");
    assert(found && is_dir, "the made directory exists");
    assert(make_directory_if_it_does_not_exist("cross-target-files/a/b"), "making it again succeeds");

    // The run sees the host's processor (`CPU` is the host's here), so only the host's own
    // instruction set families are reported, whatever the target is.
    features := SIMD.cpu_features();

    #if CPU == .ARM64 {
        assert(!features.avx2 && !features.avx512, "no x86 features on an Arm host");
    } else {
        assert(!features.sve, "no Arm features on an x86 host");
    }
}

main :: () {}
