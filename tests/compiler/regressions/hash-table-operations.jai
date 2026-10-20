// Hash tables: adding, setting, finding, removing, iterating and freeing.
#import "Basic";
#import "Hash_Table";

main :: () {
    inventory: Table(string, int);

    table_add(*inventory, "iron_ore", 64);
    table_add(*inventory, "oak_log", 32);

    found, value := table_find(*inventory, "iron_ore");

    assert(found && value == 64);
    missing, _ := table_find(*inventory, "nothing");

    assert(!missing);
    assert(inventory.count == 2);

    // table_set replaces, table_add makes a second entry.
    table_set(*inventory, "iron_ore", 128);
    table_set(*inventory, "emerald", 7);
    assert(inventory.count == 3);
    _, again := table_find(*inventory, "iron_ore");

    assert(again == 128);

    pointer := table_find_pointer(*inventory, "oak_log");

    assert(pointer != null);
    pointer.* += 16;
    _, logs := table_find(*inventory, "oak_log");

    assert(logs == 48);
    assert(table_find_pointer(*inventory, "nothing") == null);

    assert(table_contains(*inventory, "emerald"));
    assert(!table_contains(*inventory, "diamond"));

    removed, old := table_remove(*inventory, "oak_log");

    assert(removed && old == 48);
    assert(!table_contains(*inventory, "oak_log"));
    removed_again, _ := table_remove(*inventory, "oak_log");

    assert(!removed_again);
    assert(inventory.count == 2);

    entry, newly_added := find_or_add(*inventory, "gold_ingot");

    assert(newly_added && entry.* == 0);
    entry.* = 10;
    entry2, newly_added2 := find_or_add(*inventory, "gold_ingot");

    assert(!newly_added2 && entry2.* == 10);
    assert(inventory.count == 3);

    // Iteration: it is the value and it_index the key.
    total := 0;
    keys := 0;

    for inventory {
        total += it;
        keys += it_index.count;
    }

    assert(total == 128 + 7 + 10);
    assert(keys == "iron_ore".count + "emerald".count + "gold_ingot".count);

    for count, item: inventory {
        if item == "emerald" assert(count == 7);
    }

    for *inventory it.* += 1;

    _, ore := table_find(*inventory, "iron_ore");

    assert(ore == 129);

    for *qty, item: inventory {
        if item == "gold_ingot" qty.* += 10;
    }

    _, gold := table_find(*inventory, "gold_ingot");

    assert(gold == 21);

    // Entries can be removed while iterating.
    for inventory {
        if it < 50 remove it;
    }

    assert(inventory.count == 1);
    assert(table_contains(*inventory, "iron_ore"));
    assert(!table_contains(*inventory, "emerald"));
    _, kept := table_find(*inventory, "iron_ore");

    assert(kept == 129);

    table_reset(*inventory);
    assert(inventory.count == 0);
    assert(!table_contains(*inventory, "iron_ore"));
    table_add(*inventory, "again", 1);
    assert(inventory.count == 1);

    deinit(*inventory);

    // Integer keys.
    squares: Table(int, int);
    for 1..50 table_add(*squares, it, it * it);

    assert(squares.count == 50);

    for 1..50 {
        ok, sq := table_find(*squares, it);
        assert(ok && sq == it * it);
    }

    // Removing during a reverse and a by-pointer walk keeps the count in step.
    for < squares {
        if it % 2 == 0 remove it;
    }

    assert(squares.count == 25);

    for *squares {
        if it.* > 1000 remove it;
    }

    assert(squares.count == 16);

    for 1..50 {
        ok, _ := table_find(*squares, it);
        assert(ok == (it % 2 == 1 && it * it <= 1000));
    }

    deinit(*squares);

    print("ok\n");
}
