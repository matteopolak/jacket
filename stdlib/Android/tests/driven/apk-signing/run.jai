// The owned runner loads main.jai into a staged Android module for private checks.
A :: #import "Android/Toolchain";

main :: () {
    A.apk_signing_test_main();
}
