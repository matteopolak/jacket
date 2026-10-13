// Middleware and groups: timing on every request, a token check in front of /admin, and a
// route prefix with a parameter.
//
//     jaic run examples/http/middleware.jai
//     curl -i localhost:8080/hello
//     curl -i localhost:8080/admin/stats                                  # 401
//     curl -i localhost:8080/admin/stats -H 'authorization: Bearer letmein'
//     curl localhost:8080/teams/7/members/3
#import "Basic";
#import "Extensions/Http";

// Middleware gets the request and `next`, the rest of the chain. It may answer itself or let the
// request through and change the response on its way back.
timing :: (request: *Request, next: Next) -> Response {
    started := current_time_monotonic();
    response := next(request);

    add_header(*response, "X-Elapsed-Us", tprint("%", to_microseconds(current_time_monotonic() - started)));

    return response;
}

require_token :: (request: *Request, next: Next) -> Response {
    token, found := header(request, "Authorization");

    if !found return error_response(request, unauthorized("send an Authorization header"));
    if token != "Bearer letmein" return error_response(request, forbidden("that token is not accepted"));

    return next(request);
}

hello :: () -> string {
    return "hello";
}

stats :: () -> Stats {
    return .{ uptime_seconds = 42, requests = 1234 };
}

Stats :: struct {
    uptime_seconds: int;
    requests: int;
}

member :: (team: Path("team", int), id: Path("id", int)) -> string {
    return tprint("member % of team %", id.value, team.value);
}

main :: () {
    app: App;

    use(*app, timing); // wraps everything, 404s included

    get(*app, "/hello", hello);

    admin := group(*app, "/admin", require_token);

    get(admin, "/stats", stats);

    // The prefix has a parameter; handlers below it may use Path("team", ...).
    teams := group(*app, "/teams/{team}");

    get(teams, "/members/{id}", member);

    serve(*app);
}
