// A module of the tests (imported with `#import,dir "support"`): a small tree of files written
// into the test's own folder, and ways to run a search over it and read what it printed.

#import "Basic";
#import "String";
#import "File";
#import "Extensions/Testing";
#import,dir "../../search";

// The folder the test's files go in.
tree :: (t: *Test) -> string {
    return tprint("%/tree", temp_dir(t));
} @TempAllocates

// Writes `text` to `relative` under `root`, making the folders on the way.
add_file :: (root: string, relative: string, text: string) {
    path := tprint("%/%", root, relative);

    require(make_directory_if_it_does_not_exist(path_strip_filename(path), recursive = true), "the folder can be made");
    require(write_entire_file(path, text), "the file can be written");
}

// A tree used by most tests:
//
//     a.txt          "one\nTwo needle\nthree\n"
//     b.md           "needle first\nNeedle again\n"
//     sub/c.txt      "deep needle\n"
//     sub/d.bin      a NUL, then "needle"
//     .hidden/e.txt  "needle in hiding\n"
a_small_tree :: (t: *Test) -> string {
    root := tree(t);

    add_file(root, "a.txt", "one\nTwo needle\nthree\n");
    add_file(root, "b.md", "needle first\nNeedle again\n");
    add_file(root, "sub/c.txt", "deep needle\n");
    add_file(root, "sub/d.bin", "\x00needle");
    add_file(root, ".hidden/e.txt", "needle in hiding\n");

    return root;
} @TempAllocates

// The options for searching `pattern` in the folder `root`. Change fields before `run_search`.
// The paths are in temporary storage.
options_for :: (root: string, pattern: string) -> Options {
    return options_in(pattern, root);
}

// The options for searching `pattern` in `paths`, each given relative to nothing: as written.
options_in :: (pattern: string, paths: ..string) -> Options {
    list := NewArray(paths.count, string,, temp);

    for paths list[it_index] = it;

    return .{ pattern = pattern, paths = list };
}

// What the report printed, with the folder `root` taken off the front of every path. In
// temporary storage.
shown :: (report: Report, root: string) -> string {
    return replace(report.output, tprint("%/", root), "",, temp);
}

// The report's error messages joined by newlines, with `root` taken off. In temporary storage.
complaints :: (report: Report, root: string) -> string {
    text := join(..report.errors, "\n",, temp);

    return replace(text, tprint("%/", root), "",, temp);
}
