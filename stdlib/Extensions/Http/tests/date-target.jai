#import "Basic";
Http :: #import "Extensions/Http";

main :: () {
    cache: Http.Date_Cache;
    text := Http.http_date(*cache, 784111777);

    assert(text == "Sun, 06 Nov 1994 08:49:37 GMT");
    seconds, ok := Http.parse_http_date(text);

    assert(ok && seconds == 784111777);
    assert(Http.http_date(*cache, 0) == "Thu, 01 Jan 1970 00:00:00 GMT");
    assert(Http.http_date(*cache, -86400) == "Wed, 31 Dec 1969 00:00:00 GMT");
}
