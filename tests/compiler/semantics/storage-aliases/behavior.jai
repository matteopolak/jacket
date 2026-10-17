// F and R import the same original backing module instance.
#assert type_of(F.storage_alias_counter) == F.Handle && F.Handle == R.Handle;
#assert type_of(F.storage_alias_pair) == R.Pair;

#program_export "storage_alias_check"
storage_alias_check :: () -> s32 #c_call {
    if *F.storage_alias_counter != *R.storage_alias_counter return 1;
    if *F.storage_alias_pair != *R.storage_alias_pair return 3;

    F.storage_alias_counter = cast(F.Handle) 29;
    copied := F.storage_alias_counter;

    if F.storage_alias_read() != 29 || R.storage_alias_counter != xx 29 return 4;

    F.storage_alias_write(43);
    if copied != xx 29 return 9;
    if F.storage_alias_counter != xx 43 return 5;
    if F.default_read() != xx 43 || !F.classify(F.storage_alias_counter) return 6;

    F.storage_alias_pair.left = 17;
    R.storage_alias_pair.right = 19;
    if R.storage_alias_pair.left != 17 || F.storage_alias_pair.right != 19 return 7;

    F.storage_alias_counter = cast(F.Handle) 7;
    if F.storage_alias_read() != 7 return 8;

    return 0;
}

main :: () -> s32 {
    return storage_alias_check();
}
