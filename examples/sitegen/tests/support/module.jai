// A module of the tests (imported with `#import,dir "support"`): a small source tree written into
// the test's own folder, and ways to look at what a build wrote.

#import "Basic";
#import "String";
#import "File";
#import "Extensions/Testing";
#import,dir "../../site";

// A site being built: where its source and output folders are.
Fixture :: struct {
    src: string;
    out: string;
}

fixture :: (t: *Test) -> Fixture {
    return .{ src = tprint("%/src", temp_dir(t)), out = tprint("%/out", temp_dir(t)) };
} @TempAllocates(0)

// Writes `text` to `relative` in the source folder, making the folders on the way.
add_source :: (site: Fixture, relative: string, text: string) {
    path := tprint("%/%", site.src, relative);

    require(make_directory_if_it_does_not_exist(path_strip_filename(path), recursive = true), "the folder can be made");
    require(write_entire_file(path, text), "the file can be written");
}

// What `relative` in the output folder holds, or "" when it is not there. In temporary storage.
output_text :: (site: Fixture, relative: string) -> string {
    text, ok := read_entire_file(tprint("%/%", site.out, relative),, temp);

    return ifx ok then text else "";
}

output_exists :: (site: Fixture, relative: string) -> bool {
    _, ok := read_entire_file(tprint("%/%", site.out, relative),, temp);

    return ok;
}

// The file in `result.files` that starts with `prefix` and ends with `suffix`: the name a hashed
// asset was published under. "" when none matches.
published_as :: (result: Build_Result, prefix: string, suffix: string) -> string {
    for result.files {
        if begins_with(it, prefix) && ends_with(it, suffix) return it;
    }

    return "";
}

// Whether any warning contains `needle`.
warned_about :: (result: Build_Result, needle: string) -> bool {
    for result.warnings {
        if contains(it, needle) return true;
    }

    return false;
}
