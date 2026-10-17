#scope_file
#load "loaded_piece.jai";

file_private := 23;

#scope_export
#import "Basic";

main :: () {
    assert(exported_from_loaded_file == 37, "loaded file starts in export scope");
    assert(file_private == 23, "file-local declaration remains visible in its file");
    print("ok\n");
}
