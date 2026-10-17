Http :: #import "Extensions/Http";

// Reviewed native adapter only; a WASM request must diagnose the missing export.
main :: () {
    cache: Http.Date_Cache;
    text := Http.http_date_now(*cache); // jailint: allow(unused_variable) the call itself is what must be diagnosed
}
