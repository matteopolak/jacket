// Shared echo setup for conformance peers and benchmarks.
#import "Basic";
#import "Extensions/WebSocket";
Http :: #import "Extensions/Http";
Io :: #import "Extensions/Io";
Tls :: #import "Extensions/Tls";
#load "../../../Tls/tests/fixtures/certs.jai";

config: Ws_Config;

fixture :: (name: string) -> string {
    for DATA if it.name == name return it.text;
    return "";
}

echo :: (connection: *Ws_Connection, data: *void) {
    while true {
        event := ws_read(connection);
        if event.kind == .CLOSE || event.kind == .ERROR return;
        assert(event.kind == .MESSAGE);
        if ws_write(connection, event.data, event.opcode).kind != .NONE return;
    }
}

route :: (request: *Http.Request) -> Http.Response {
    return ws_upgrade(request, echo, null, config);
}
