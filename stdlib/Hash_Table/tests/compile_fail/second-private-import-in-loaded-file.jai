// A loaded file's file-scope module import is not visible to the loading file.
#import "Basic";
#load "support/named-private-file.jai";

main :: () {
    path, ok := Hidden_File.get_absolute_path(".");
    assert(ok);
    free(path);
}
