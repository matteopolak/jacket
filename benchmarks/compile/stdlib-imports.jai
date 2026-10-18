// Standard library: many modules imported, so loading and checking them dominates. Most are not
// called on purpose: the cost under test is the import.
// jailint: allow-file(unused_import)
#import "Basic";
#import "Math";
#import "String";
#import "Hash_Table";
#import "Sort";
#import "Random";
#import "File";
#import "File_Utilities";
#import "Bit_Operations";
#import "Hash";
#import "Reflection";
#import "Command_Line";
#import "Process";
#import "System";
#import "Unicode";
#import "Text_File_Handler";
#import "Thread";

main :: () {
    table: Table(string, int);

    table_set(*table, "answer", 42);
    values := int.[3, 1, 2];

    quick_sort(values, (a, b) => a - b);
    random_seed(7);
    mask: u64 = 255;

    print("% % % % %\n", sqrt(2.0), contains("shout", "out"), values[0], table_find_pointer(*table, "answer").*, popcount(mask));
}
