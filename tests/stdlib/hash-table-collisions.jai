Basic :: #import "Basic";
Tables :: #import "Hash_Table" (COUNT_COLLISIONS=true);
constant_hash :: (key:s64)->u32 { return 9; }
main :: () -> s64 {
    table:Tables.Table(s64,s64,constant_hash);
    Tables.init(*table,32);
    defer Tables.deinit(*table);
    Tables.table_add(*table,1,10); Tables.table_add(*table,1,20); Tables.table_add(*table,2,30);
    if table.count!=3 || table.slots_filled!=3 return 1;
    duplicates:=Tables.table_find_multiple(*table,1);
    if duplicates.count!=2 || duplicates[0]+duplicates[1]!=30 return 2;
    removed,value:=Tables.table_remove(*table,1);
    if !removed || table.count!=2 || table.slots_filled!=3 return 3;
    Tables.table_add(*table,3,40);
    if table.count!=3 || table.slots_filled!=3 return 4;
    for i:4..80 Tables.table_set(*table,i,i*10);
    for i:4..80 {
        ok,found:=Tables.table_find(*table,i);
        if !ok || found!=i*10 return 5;
    }
    pointer,newly:=Tables.find_or_add(*table,99);
    if !newly return 6;
    pointer.*=123;
    ok,found:=Tables.table_find(*table,99);
    if !ok || found!=123 return 7;
    if table.add_collisions==0 || table.find_collisions==0 return 8;
    Tables.table_reset(*table);
    if table.count!=0 || table.slots_filled!=0 || table.add_collisions!=0 return 9;
    return 0;
}
