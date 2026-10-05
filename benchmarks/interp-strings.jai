// Interpreter: string building, formatting and hash tables.
#import "Basic";
#import "Hash_Table";
#import "String";

main :: () {
    table: Table(string, int);
    builder: String_Builder;
    for 0..19_999 {
        key := tprint("key-%-%", it, it * 0.5);
        table_set(*table, key, it);
        append(*builder, key);
    }
    found := 0;
    for 0..19_999 {
        ok, value := table_find(*table, tprint("key-%-%", it, it * 0.5));
        if ok && value == it found += 1;
    }
    text := builder_to_string(*builder);
    print("% % %\n", found, text.count, split(text, "-").count);
}
