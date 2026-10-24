#import "Basic";

#scope_file

// Reached through the code `#insert` builds from a string.
made_by_insert :: () {
    print("hi\n");
}

#scope_export

main :: () {
    #insert "made_by_insert();";
}
