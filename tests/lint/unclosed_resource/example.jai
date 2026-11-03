#import "Basic";
#import "File";

is_readable :: (path: string) -> bool {
    file, ok := file_open(path);
    if !ok return false;

    return true;
}

main :: () {
    print("%\n", is_readable("notes.txt"));
}
