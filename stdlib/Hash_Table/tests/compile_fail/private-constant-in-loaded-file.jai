// A loaded file's file-scope constant is not visible to the loading file.
#import "Basic";
#load "support/named-private-file.jai";

main :: () {
    print("%", private_number);
}
