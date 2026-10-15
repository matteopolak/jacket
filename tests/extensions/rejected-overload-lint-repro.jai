// No Http/Json dependency: a rejected generic overload must not become a real
// body-check obligation during the IDE/lint audit.
#import "Basic";

Client :: struct {
    marker: int;
}

App :: struct {
    marker: int;
}

called: bool;

post :: (client: *Client, payload: $V) {
    #assert V == int "rejected Client candidate was checked";
    client.marker = payload;
}

post :: (app: *App, $handler: $H) {
    handler();
    app.marker = 1;
}

handler :: () {
    called = true;
}

main :: () {
    app: App;
    post(*app, handler);
    assert(called && app.marker == 1);
}
