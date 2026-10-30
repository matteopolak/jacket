// test: skip_on(.WASM, reason = "Http.http_date_now exists only for native targets")
#import "Basic";
Http :: #import "Extensions/Http";

main :: () {
    cache: Http.Date_Cache;
    text := Http.http_date_now(*cache);
    seconds, ok := Http.parse_http_date(text);
    assert(ok && seconds > 0);
}
