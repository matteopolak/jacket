// MacOS_Bundler: build a bundle (without an icon, so `sips` is not needed) in a scratch
// directory and check its layout, the copied resources and the generated Info.plist.
#import "Basic";
#import "File";
#import "File_Utilities";
#import "String";
#import "MacOS_Bundler";

SCRATCH :: "/tmp/jaic_macos_bundler_test";

main :: () {
    delete_directory(SCRATCH);
    assert(make_directory_if_it_does_not_exist(tprint("%/assets/sub", SCRATCH), recursive = true));
    assert(write_entire_file(tprint("%/demo_exe", SCRATCH), "binary"));
    assert(write_entire_file(tprint("%/assets/a.txt", SCRATCH), "A"));
    assert(write_entire_file(tprint("%/assets/sub/b.txt", SCRATCH), "B"));
    previous := get_working_directory();
    assert(set_working_directory(SCRATCH));

    // Resources keep their folder name, and the plist has no icon entry.
    bundle, resources := create_app_bundle("Demo", "demo_exe", "", "assets/", false, true);
    assert(bundle == "Demo.app");
    assert(resources == "Demo.app/Contents/Resources");
    executable, ok := read_entire_file("Demo.app/Contents/MacOS/Demo");
    assert(ok && executable == "binary");
    nested := read_entire_file("Demo.app/Contents/Resources/assets/sub/b.txt");
    assert(nested == "B");
    plist := read_entire_file("Demo.app/Contents/Info.plist");
    assert(contains(plist, "<plist version=\"1.0\">"));
    assert(!contains(plist, "CFBundleIconFile"));

    // Flattened: the directory's contents go straight into Resources; no plist written.
    create_app_bundle("Flat", "demo_exe", "", "assets", true, false);
    assert(read_entire_file("Flat.app/Contents/Resources/a.txt") == "A");
    assert(file_exists("Flat.app/Contents/Resources/sub/b.txt"));
    assert(!file_exists("Flat.app/Contents/Info.plist"));

    // The plist helpers escape XML and reject unsupported value types.
    builder: String_Builder;
    assert(plist_write_key_value(*builder, "Name", "a<b&c"));
    assert(plist_write_key_value(*builder, "Flag", true));
    assert(!plist_write_key_value(*builder, "Count", 3));
    text := builder_to_string(*builder);
    assert(contains(text, "<key>Name</key>\n<string>a&lt;b&amp;c</string>\n"));
    assert(contains(text, "<true/>"));

    set_working_directory(previous);
    delete_directory(SCRATCH);
    print("ok\n");
}
