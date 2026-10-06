#import "Basic";
#import "Math";
Strings :: #import "String";
#import "Hash_Table";

main :: () {
    table: Table(string, int);
    table_set(*table, "one", 1);
    print("% %\n", sqrt(2.0), Strings.trim("  x  "));
    // Only used in code for another platform: still counted.
    #if OS == .WINDOWS {
        print("%\n", Strings.to_upper_copy("windows"));
    }
}
