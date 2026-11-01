Mail_Job :: struct {
    host: string;
    port: s64;
    recipient: string;
    body: string;
}

// Mail is blocking, so one OS worker owns each immutable job, outside the network scheduler.
deliver_mail :: (group: *Thread_Group, thread: *Thread, raw: *void) -> Thread_Continue_Status {
    job := cast(*Mail_Job) raw;
    smtp := Mail.Mail_Server.{ host = job.host, port = job.port, timeout_milliseconds = 2000 };
    message := Mail.Mail_Message.{ from = "chat@localhost", to = string.[job.recipient], subject = "Chat mention digest", body = job.body };

    if !Mail.mail_send(smtp, message) print("chat: digest delivery failed for %\n", job.recipient, to_standard_error = true);

    free(job.host);
    free(job.recipient);
    free(job.body);
    free(job);
    reset_temporary_storage();

    return .CONTINUE;
}

send_digests :: (server: *Server) {
    if server.options.smtp_host.count == 0 return;

    // Bound the worker queue: leave pending digests for the next tick while it is busy.
    server.mail_inflight -= get_completed_work(*server.mail_group).count;
    if server.mail_inflight > 0 return;

    for server.users {
        if it.pending.count == 0 continue;

        body: String_Builder;

        for pending: it.pending {
            append(*body, pending);
            free(pending);
        }

        it.pending.count = 0;
        job := New(Mail_Job);
        job.* = .{ copy_string(server.options.smtp_host), server.options.smtp_port, copy_string(it.email), builder_to_string(*body) };
        free_buffers(*body);
        add_work(*server.mail_group, job);
        server.mail_inflight += 1;
    }
}

publish_stats :: (server: *Server, recent: s64) {
    connected, writer := Shm.writer_connect(server.options.channel, name_not_found_is_quiet = true);
    if !connected return;

    defer Shm.writer_disconnect(*writer);
    text := tprint("connections=% messages/s=% total=%", server.connections, recent, server.messages);
    reserved, message := Shm.writer_reserve_message(*writer, text.count);

    if !reserved return;

    memcpy(message.user_write_pointer, text.data, text.count);
    Shm.writer_commit_message(*writer, message);
} @TempAllocates

read_stats :: (name: string) {
    created, reader := Shm.reader_create_channel(name, 4096);
    if !created {
        print("chat: channel already has a reader or cannot be created\n");
        exit(1);
    }

    defer Shm.reader_release_channel(*reader);

    for 0..39 {
        found, fatal, message := Shm.reader_poll_for_message(*reader);
        if fatal {
            print("chat: damaged stats channel\n");
            exit(1);
        }

        if found {
            print("%\n", string.{ message.user_read_bytes, message.user_read_pointer });
            Shm.reader_done_with_message(*message);
            return;
        }

        sleep_milliseconds(100);
    }

    print("chat: no stats publisher\n");
    exit(1);
}

// A deliberately minimal local SMTP sink. It accepts one message and records DATA, including
// the MIME headers, without interpreting them. Only the loopback address is configurable by port.
run_sink :: (port: s64, path: string) {
    listener, error := Io.tcp_listen(tprint("127.0.0.1:%", port));
    if error != .NONE exit(1);

    defer Io.close(listener);
    print("SMTP READY\n");
    tcp, accepted := Io.accept(listener);
    if accepted != .NONE exit(1);

    link := Link.{ tcp = tcp };

    link_prepare(*link);
    defer link_close(*link);
    send_line(*link, "220 localhost chat sink");
    body: String_Builder;

    defer free_buffers(*body);
    in_data := false;

    Io.with_deadline(5000, #code {
        while true {
            line, received := Io.read_line(*link.reader);
            if received != .NONE exit(1);

            if in_data {
                if line == "." {
                    text := builder_to_string(*body);

                    defer free(text);
                    if !write_entire_file(path, text) exit(1);

                    send_line(*link, "250 stored");
                    in_data = false;
                } else print_to_builder(*body, "%\n", line);
            } else if begins_with(line, "EHLO") || begins_with(line, "HELO") {
                send_line(*link, "250 localhost");
            } else if line == "DATA" {
                send_line(*link, "354 end with dot");
                in_data = true;
            } else if line == "QUIT" {
                send_line(*link, "221 bye");
                break;
            } else send_line(*link, "250 ok");
        }
    });
}
