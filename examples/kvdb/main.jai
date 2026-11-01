// kvdb: a small key-value database. An append-only log with checksummed records, an in-memory
// index, compaction, and a TCP server for it. README.md is the guide.
//
//     jaic examples/kvdb/main.jai -exe kvdb
//     ./kvdb set greeting hello
//     ./kvdb get greeting
//     ./kvdb scan gree
//     ./kvdb export
//     ./kvdb serve --listen 127.0.0.1:7878 &
//     ./kvdb remote --address 127.0.0.1:7878 SET name Ada
//
// The data file is `kvdb.log` in the current folder; `--file` (or KVDB_FILE) picks another.

#import "Basic";
#import "String";
#import "Base64";
Io :: #import "Extensions/Io";
Args :: #import "Extensions/Args";
#import,dir "store";
#import,dir "protocol";

Command :: enum {
    GET;
    SET;
    DEL;
    SCAN;
    COMPACT;
    STATS;
    EXPORT;
    SERVE;
    REMOTE;
}

Key_Args :: struct {
    key: string; @positional @required
}

Set_Args :: struct {
    key: string; @positional @required
    value: string; @positional @required
}

Scan_Args :: struct {
    prefix: string; @positional
}

Serve_Args :: struct {
    listen: string = "127.0.0.1:7878"; @short=l @value=ADDRESS
}

Remote_Args :: struct {
    address: string = "127.0.0.1:7878"; @short=a @value=ADDRESS
    request: [..] string; @positional @required
}

No_Args :: struct {}

Cli :: struct {
    file: string = "kvdb.log"; @short=f @env=KVDB_FILE @value=PATH @global
    command: Command; @subcommand
    get: Key_Args; @subcommand
    set: Set_Args; @subcommand
    del: Key_Args; @subcommand
    scan: Scan_Args; @subcommand
    compact: No_Args; @subcommand
    stats: No_Args; @subcommand
    export: No_Args; @subcommand
    serve: Serve_Args; @subcommand
    remote: Remote_Args; @subcommand
}

HELP :: Args.Help(Cli).{
    about = "A small key-value database: an append-only log, an in-memory index, a TCP server.",
    version = "0.1.0",
    file = "The data file",
    get = .{ about = "Print the value of a key", key = "The key" },
    set = .{ about = "Store a value", key = "The key", value = "The value" },
    del = .{ about = "Remove a key", key = "The key" },
    scan = .{ about = "List the keys that start with a prefix", prefix = "The prefix (default: every key)" },
    compact = .{ about = "Rewrite the log with only the live records" },
    stats = .{ about = "Show the size of the store" },
    export = .{ about = "Print every key with its value in Base64" },
    serve = .{ about = "Serve the store over TCP", listen = "Address to listen on" },
    remote = .{ about = "Send one request to a running server", address = "The server's address", request = "The request words, e.g. SET name Ada" },
};

main :: () {
    cli, _ := Args.parse(Cli, HELP);

    // A server and a client run on the network scheduler; the local commands need none.
    if cli.command == .SERVE {
        Io.io_run(run_server, cli);
    } else if cli.command == .REMOTE {
        Io.io_run(run_remote, cli);
    } else {
        exit(run_local(cli));
    }
}

fail :: (message: string, args: ..Any) -> s32 {
    print("kvdb: %\n", tprint(message, ..args), to_standard_error = true);

    return 1;
}

// Opens the data file, saying so when a torn record had to be cut off.
open_store :: (store: *Store, path: string) -> bool {
    opened := store_open(store, path);

    if opened.status == {
        case .OK;
            return true;

        case .RECOVERED;
            print("kvdb: cut % bytes of an unfinished record from the end of %\n", opened.dropped_bytes, path, to_standard_error = true);

            return true;

        case .CORRUPT;
            fail("% is damaged: the record at byte % fails its checksum", path, opened.bad_offset);

        case;
            fail("cannot open %", path);
    }

    return false;
}

run_local :: (cli: Cli) -> s32 {
    store: Store;

    if !open_store(*store, cli.file) return 1;

    defer store_close(*store);

    if cli.command == {
        case .GET;
            value, found := store_get(*store, cli.get.key);

            if !found return 1;

            print("%\n", value);

        case .SET;
            status := store_put(*store, cli.set.key, cli.set.value);

            if status != .OK return fail("cannot store that key and value (%)", status);

        case .DEL;
            status := store_delete(*store, cli.del.key);

            if status == .MISSING return 1;
            if status != .OK return fail("cannot delete (%)", status);

        case .SCAN;
            for store_scan(*store, cli.scan.prefix) print("%\t%\n", it.key, it.value);

        case .COMPACT;
            result := store_compact(*store);

            if !result.ok return fail("compaction failed; the log is unchanged");

            print("% -> % bytes\n", result.bytes_before, result.bytes_after);

        case .STATS;
            print("keys %\nrecords %\nlog bytes %\nlive bytes %\n", store.index.count, store.log_records, store.log_size, store.live_bytes);

        case .EXPORT;
            for store_scan(*store, "") {
                encoded := base64_encode(it.value);
                defer free(encoded);

                print("%\t%\n", it.key, encoded);
            }

        case;
            return fail("not a local command");
    }

    return 0;
}

run_server :: (cli: Cli) {
    store: Store;

    if !open_store(*store, cli.file) exit(1);

    defer store_close(*store);

    server := Server.{ store = *store };
    error: Io.Io_Error;

    server.listener, error = Io.tcp_listen(cli.serve.listen);

    if error != .NONE exit(fail("cannot listen on %: %", cli.serve.listen, error));

    address, _ := Io.local_address(server.listener);

    print("kvdb: serving % on %\n", cli.file, Io.address_to_string(address));

    serve(*server);

    print("kvdb: stopped after % requests\n", server.served);
}

run_remote :: (cli: Cli) {
    client, error := connect_client(cli.remote.address);

    if error != .NONE exit(fail("cannot connect to %: %", cli.remote.address, error));

    defer close_client(*client);

    reply, call_error := call(*client, join(..cli.remote.request, " "));

    defer free_reply(reply);

    for reply print("%\n", it);

    if call_error != .NONE exit(fail("the connection failed: %", call_error));
    if reply.count > 0 && begins_with(reply[0], "ERR") exit(1);
}
