#import "Basic";
#import "File";
main :: () { f, ok := file_open("/tmp/x", for_writing=true); file_write(*f, null, -1); file_close(*f); file_close(*f); }
