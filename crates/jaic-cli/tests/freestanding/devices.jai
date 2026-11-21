registers: [2] u32;
table: [3] u32 = .[7, 8, 9]; @Section(".probe_table")

main :: () {
    volatile_store(*registers[0], 41);
    volatile_store(*registers[0], volatile_load(*registers[0]) + 1);
    send_event();
    wait_for_event();
    data_sync_barrier();
    data_memory_barrier();
    instruction_barrier();
    print("% %\n", volatile_load(*registers[0]), table[1]);
}
