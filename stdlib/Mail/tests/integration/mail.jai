#import "Basic";
M :: #import "Mail";

errors: s64;

capture :: (message: string, data: *void, info: Log_Info) {
    assert(message.count > 0 && (info.common_flags & .ERROR) != 0);
    errors += 1;
}

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 4);
    assert(type_info(M.Mail_Server).name == "Mail_Server" && type_info(M.Mail_Message).name == "Mail_Message");
    port, valid, rest := string_to_int(args[1], 10, s64);

    assert(valid && rest.count == 0 && port > 0 && port <= 65535);
    smtp: M.Mail_Server;
    smtp.host = "127.0.0.1";
    smtp.port = port;
    smtp.enable_ssl = args[3] == "tls";

    if args[3] == "auth" {
        smtp.username = "portable-user";
        smtp.password = "portable-password";
    }

    message: M.Mail_Message;
    message.from = "<sender@example.invalid>";
    message.reply_to = "<reply@example.invalid>";
    message.to = .["<to@example.invalid>", "<second@example.invalid>"];
    message.cc = .["<cc@example.invalid>"];
    message.bcc = .["<bcc@example.invalid>"];
    message.subject = "Portable stdlib";
    message.body = "portable π body\r\n";
    message.html = args[3] != "plain";
    message.attachments = .[args[2]];
    previous := context.logger;

    context.logger = capture;
    defer context.logger = previous;
    invalid := message;
    invalid.subject = "invalid\r\nInjected: header";
    assert(!M.mail_send(smtp, invalid) && errors > 0);
    errors = 0;
    sent := M.mail_send(smtp, message);
    if args[3] == "reject" || args[3] == "tls" assert(!sent && errors > 0);
    else assert(sent && errors == 0);

    print("ok\n");
}
