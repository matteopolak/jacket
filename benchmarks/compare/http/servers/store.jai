// The record every server of the comparison returns, how many there are, and the settings the driver
// (tools/bench/http_compare.py) passes in the environment.
ITEMS :: 1000;

Item :: struct {
    id: int;
    name: string;
    qty: int;
    active: bool;
    note: string;
}

fill_items :: (items: *[ITEMS] Item) {
    for *items.* {
        it.id = it_index + 1;
        it.name = tprint("item-%", it_index + 1);
        it.name = copy_string(it.name);
        it.qty = it_index % 100;
        it.active = it_index % 2 == 0;
        it.note = "xxxxxxxxxxxxxxxxxxxx";
    }
}

// The same bytes the Json module writes for an `Item`.
item_json :: (item: Item) -> string {
    return tprint("{\"id\":%,\"name\":\"%\",\"qty\":%,\"active\":%,\"note\":\"%\"}", item.id, item.name, item.qty, item.active, item.note);
}

env_int :: (name: string, fallback: int) -> int {
    text := environment_variable(name);
    if !text return fallback;

    value, ok := parse_int(*text);
    if !ok return fallback;

    return value;
}
