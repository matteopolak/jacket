// The comparison API on smari/jai-simplehttp with its default settings (regular-expression routes):
// GET /items/{id} returns the stored item as JSON. BENCH_PORT comes from tools/bench/http_compare.py.
// The server is single-threaded.
#import "Basic";
#import "String";
#import "Extensions/Environment";
Simple :: #import "smari--jai-simplehttp"(USE_LOGGING = false);

#load "store.jai";

items: [ITEMS] Item;

get_item :: (request: *Simple.HTTPRequest) -> Simple.HTTPResponse {
    response: Simple.HTTPResponse;
    id := string_to_int(request.url_captures[1]);

    if id < 1 || id > ITEMS {
        response.status = 404;
        response.body = copy_string("not found");
        return response;
    }

    response.status = 200;
    response.body = copy_string(item_json(items[id - 1]));
    Simple.http_response_header_add(*response, "Content-Type", "application/json");
    Simple.http_response_header_add(*response, "Content-Length", tprint("%", response.body.count));

    return response;
}

main :: () {
    fill_items(*items);

    server := Simple.http_server_setup("127.0.0.1", cast(u16) env_int("BENCH_PORT", 8080));

    Simple.http_register_url(*server, "^/items/([0-9]+)$", .GET, get_item);
    error := Simple.http_server_run_single_threaded(*server);

    print("server stopped: %\n", error);
}
