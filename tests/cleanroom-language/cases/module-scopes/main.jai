#import "Basic";

Visible_Module :: #import "Visible_Module";

main :: () {
    assert(Visible_Module.public_in_root == 13, "export-scope declaration is importable");
    assert(Visible_Module.public_from_loaded_file == 19, "loaded file has independent export scope");
    print("ok\n");
}
