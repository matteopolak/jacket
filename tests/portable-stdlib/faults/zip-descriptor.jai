#import "Basic";
Z :: #import "Zip_File_Directory";
S :: #import "String";

main :: () {
    path := sprint("%/../../../target/portable-tests/zip/descriptor.zip", S.path_strip_filename(#file));
    defer free(path);
    Z.load_zip_directory(path);
    assert(false, "unsupported data descriptor returned");
}
