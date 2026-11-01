// A folder name that ends in a slash is the same folder, also when its parents have to be made.
#import "Basic";
F :: #import "File";

main :: () {
    assert(F.make_directory_if_it_does_not_exist("trailing-slash-test/a/b/", true));
    assert(F.make_directory_if_it_does_not_exist("trailing-slash-test/a/b/", true));
    assert(F.make_directory_if_it_does_not_exist("trailing-slash-test/c/", true));
    assert(!F.make_directory_if_it_does_not_exist("trailing-slash-test/x/y/"));

    assert(F.delete_directory("trailing-slash-test/a/b"));
    assert(F.delete_directory("trailing-slash-test/a"));
    assert(F.delete_directory("trailing-slash-test/c"));
    assert(F.delete_directory("trailing-slash-test"));

    print("ok\n");
}
