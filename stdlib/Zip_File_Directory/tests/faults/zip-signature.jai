#import "Basic";
Z :: #import "Zip_File_Directory";
S :: #import "String";

main :: () {
    path := sprint("%/../../../../target/portable-tests/zip/signature.zip", S.path_strip_filename(#file));
    defer free(path);
    Z.load_zip_directory(path);
    assert(false, "invalid directory signature returned");
}
