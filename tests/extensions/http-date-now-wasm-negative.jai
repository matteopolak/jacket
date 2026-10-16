Http :: #import "Extensions/Http";

// Reviewed native adapter only; a WASM request must diagnose the missing export.
main :: () {
    cache: Http.Date_Cache;
    text := Http.http_date_now(*cache);
}
