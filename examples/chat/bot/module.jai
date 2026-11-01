// A room bot. No side effects beyond replies to !ping, !echo and !help.
#import "Basic";
#import "String";
Chat :: #import,dir "../chat";
Io :: #import "Extensions/Io";

run_bot :: (options: Chat.Client_Options, duration: s64) {
    link: Chat.Link;

    defer Chat.link_close(*link);
    connected := false;

    Io.with_deadline(5000, #code {
        connected = Chat.client_open(*link, options);
    });
    if !connected exit(1);

    print("BOT READY\n");
    started := seconds_since_init();
    while !Io.interrupted() && (duration < 0 || (seconds_since_init() - started) * 1000 < cast(float64) duration) {
        line: string;
        error: Io.Io_Error;

        Io.with_deadline(200, #code {
            line, error = Io.read_line(*link.reader);
        });
        if error == .TIMED_OUT continue;
        if error != .NONE return;
        if !begins_with(line, "MSG ") continue;

        _, _, rest := split_from_left(line, #char " ");
        _, _, authored := split_from_left(rest, #char " ");
        _, nick, text := split_from_left(authored, #char " ");

        if nick == options.nick continue;

        answer: string;
        if text == "!ping" answer = "pong";
        else if text == "!help" answer = "commands: !ping !echo TEXT !help";
        else if begins_with(text, "!echo ") answer = slice(text, 6, text.count - 6);
        if answer.count > 0 && !Chat.send_line(*link, tprint("SAY %", answer)) exit(1);

        reset_temporary_storage();
    }
}
