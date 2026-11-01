// Multi-room fiber chat. See README.md for the protocol and smoke test.
Io :: #import "Extensions/Io";
Args :: #import "Extensions/Args";
#import,dir "chat";
Bot :: #import,dir "bot";

Command :: enum {
    SERVE;
    CLIENT;
    BOT;
    LOAD;
    STATS;
    SMTP_SINK;
}

Serve_Args :: struct {
    listen: string = "127.0.0.1:7979";
    file: string = "chat.log";
    history: s64 = 20; @range(1,100)
    rate: s64 = 5; @range(1,100000)
    channel: string = "jacket-chat";
    tls_cert: string;
    tls_key: string;
    smtp_host: string;
    smtp_port: s64 = 2525; @range(1,65535)
}

Client_Args :: struct {
    address: string = "127.0.0.1:7979";
    roots: string;
    nick: string = "guest";
    room: string = "lobby";
    script: string;
}

Bot_Args :: struct {
    address: string = "127.0.0.1:7979";
    roots: string;
    nick: string = "bot";
    room: string = "lobby";
    duration: s64 = -1; @value=MS
}

Load_Args :: struct {
    address: string = "127.0.0.1:7979";
    roots: string;
    clients: s64 = 4; @range(1,64)
    rounds: s64 = 10; @range(1,10000)
}

Stats_Args :: struct {
    channel: string = "jacket-chat";
}

Sink_Args :: struct {
    port: s64 = 2525; @range(1,65535)
    file: string = "digest.eml";
}

Cli :: struct {
    command: Command; @subcommand
    serve: Serve_Args; @subcommand
    client: Client_Args; @subcommand
    bot: Bot_Args; @subcommand
    load: Load_Args; @subcommand
    stats: Stats_Args; @subcommand
    smtp_sink: Sink_Args; @subcommand
}

HELP :: Args.Help(Cli).{
    about = "Fiber chat: rooms, history, terminal client, bot, load, stats and local SMTP sink.",
    serve = .{ about = "Serve chat", listen = "TCP address", file = "Append-only history log", history = "Messages retained per room", rate = "Commands per second, burst 20", channel = "Stats channel", tls_cert = "Certificate PEM", tls_key = "Private key PEM", smtp_host = "Digest SMTP host (empty disables)", smtp_port = "SMTP port" },
    client = .{ about = "Join chat", address = "Server address", roots = "TLS CA PEM (enables TLS; name localhost)", nick = "Nickname", room = "Initial room", script = "Read protocol commands from file" },
    bot = .{ about = "Reply to !ping, !echo, !help", address = "Server address", roots = "TLS CA PEM", nick = "Bot nickname", room = "Room", duration = "Run for milliseconds (-1 forever)" },
    load = .{ about = "Concurrent clients, ACK latency and throughput", address = "Server address", roots = "TLS CA PEM", clients = "Concurrent clients", rounds = "Messages per client" },
    stats = .{ about = "Read one live shared-memory snapshot", channel = "Channel name; one reader at a time" },
    smtp_sink = .{ about = "Accept one email on loopback", port = "Loopback SMTP port", file = "Write message here" },
};

main :: () {
    cli, _ := Args.parse(Cli, HELP);

    Io.catch_interrupts();

    if cli.command == .STATS {
        read_stats(cli.stats.channel);
        return;
    }

    Io.io_run(dispatch, cli);
}

dispatch :: (cli: Cli) {
    if cli.command == {
        case .SERVE;
            s := cli.serve;

            run_server(.{ listen = s.listen, file = s.file, history = s.history, rate = cast(float64) s.rate, channel = s.channel, tls_cert = s.tls_cert, tls_key = s.tls_key, smtp_host = s.smtp_host, smtp_port = s.smtp_port });
        case .CLIENT;
            c := cli.client;

            run_client(.{ address = c.address, roots = c.roots, nick = c.nick, room = c.room, script = c.script });
        case .BOT;
            b := cli.bot;

            Bot.run_bot(.{ address = b.address, roots = b.roots, nick = b.nick, room = b.room }, b.duration);
        case .LOAD;
            l := cli.load;

            run_load(.{ address = l.address, roots = l.roots, clients = l.clients, rounds = l.rounds });
        case .SMTP_SINK;
            run_sink(cli.smtp_sink.port, cli.smtp_sink.file);
        case .STATS;
    }
}
