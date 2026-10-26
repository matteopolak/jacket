// The comparison API on the Http server of Jacket: GET /items/{id} returns the stored item as JSON.
// BENCH_PORT and BENCH_THREADS (default 4) come from tools/bench/http_compare.py.
#import "Basic";
#import "String";
#import "Extensions/Http";
#import "Extensions/Environment";

#load "store.jai";

Store :: struct {
    items: [ITEMS] Item;
}

get_item :: (id: Path("id", int), store: *State(Store)) -> Item, Http_Error {
    if id.value < 1 || id.value > ITEMS return .{}, not_found("No item with id %", id.value);
    return store.items[id.value - 1], .{};
}

main :: () {
    store: Store;

    fill_items(*store.items);

    app: App;
    app.state = *store;
    app.config.address = tprint(":%", env_int("BENCH_PORT", 8080));
    app.config.threads = env_int("BENCH_THREADS", 4);
    get(*app, "/items/{id}", get_item);
    error := serve(*app);

    if error.kind != .NONE {
        print("%\n", error_text(error));
        exit(1);
    }
}
