#import "Basic";
#import "File";
main :: () { write_entire_file("", "x"); read_entire_file("/"); make_directory_if_it_does_not_exist("/a/b/c/d", recursive=true); file_delete("/"); }
