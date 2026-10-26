// The comparison API on rluba/hyperserve: GET /items/{id} returns the stored item as JSON.
// BENCH_PORT and BENCH_THREADS (default 4) come from tools/bench/http_compare.py.
#import "Basic";
#import "String";
#import "Extensions/Environment";
Hyper :: #import "rluba--hyperserve";

#load "store.jai";

items: [ITEMS] Item;

Path_Id :: struct {
    id: u64;
}

get_item :: (request: *Hyper.Request, parameters: *Hyper.Request_Parameters(Path_Id)) {
    id := cast(int) parameters.path.id;

    if id < 1 || id > ITEMS {
        Hyper.respond("not found", 404);
        return;
    }

    Hyper.respond(item_json(items[id - 1]), content_type = "application/json");
}

main :: () {
    fill_items(*items);

    server: Hyper.Server;
    server.verbose = .NONE;

    ok := Hyper.add_route(*server, .GET, "/items/{id}", get_item, auth = .OFF);
    if !ok exit(1);

    port := cast(u16) env_int("BENCH_PORT", 8080);

    ok = Hyper.start(*server, port = port, num_threads = cast(s32) env_int("BENCH_THREADS", 4));
    if !ok exit(1);

    while server.state != .STOPPED {
        Hyper.update(*server);
        reset_temporary_storage();
    }
}
