// test: only(.LINUX)
// `readdir` returns the 64-bit directory record on every Linux CPU, including 32-bit ones, where
// the binding links `readdir64`: the entry names and kinds come out right.
#import "Basic";
#import "POSIX";

#assert offset_of(dirent, "d_type") == 18;
#assert offset_of(dirent, "d_name") == 19;

DT_DIRECTORY :: 4;

main :: () {
    directory := opendir(".");

    assert(directory != null, "opendir");
    found_dot := false;
    found_dotdot := false;

    while true {
        entry := readdir(directory);

        if !entry break;

        name := to_string(*entry.d_name[0]);

        if name == "." {
            found_dot = true;
            assert(entry.d_type == DT_DIRECTORY, "'.' is a directory, type %", entry.d_type);
        }

        if name == ".." found_dotdot = true;
    }

    closedir(directory);
    assert(found_dot && found_dotdot, "'.' and '..' are listed");
    print("ok\n");
}
