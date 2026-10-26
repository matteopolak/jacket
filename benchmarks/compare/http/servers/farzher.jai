// The comparison API on farzher/Jai-HTTP-Server: GET /items/:id returns the stored item as JSON.
// BENCH_PORT comes from tools/bench/http_compare.py. The library has an epoll backend (Linux) and a
// Windows backend only, so this program does not build on macOS; the driver records the compiler's error.
#import "Basic";
#import "String";
#import "Extensions/Environment";
#import "farzher--Jai-HTTP-Server";

#load "store.jai";

items: [ITEMS] Item;

get_item :: (request: *Request) {
    id := string_to_int(request.params["id"]);

    if id < 1 || id > ITEMS {
        send_response(request, "not found", 404);
        return;
    }

    send_json(request, item_json(items[id - 1]));
}

main :: () {
    fill_items(*items);

    http_listen(port = cast(u16) env_int("BENCH_PORT", 8080));
    get("/items/:id", get_item);

    while true sleep_milliseconds(1000);
}
