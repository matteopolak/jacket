main :: () {
    state: s32;
    count: s64;

    one_time_init(*state, #code {
        count += 1;
    });
    one_time_init(*state, #code {
        count += 10;
    });
    if count != 1 debug_break();

    write_string("ok\n");
}
