// Hash_Table from the compiler's own modules: inserts, then lookups that hit and miss.
#import "Basic";
#import "Hash_Table";

main :: () {
    table: Table(int, int);

    for 0..2_000_000 - 1 table_set(*table, it * 7919 % 4_000_037, it);

    hits, sum := 0, 0;

    for 0..4_000_000 - 1 {
        value := table_find_pointer(*table, it);
        if value {
            hits += 1;
            sum += value.*;
        }
    }

    print("% %\n", hits, sum);
}
