Server_Options :: struct {
    listen: string = "127.0.0.1:7979";
    file: string = "chat.log";
    history: s64 = 20;
    rate: float64 = 5;
    channel: string = "jacket-chat";
    tls_cert: string;
    tls_key: string;
    smtp_host: string;
    smtp_port: s64 = 2525;
}

User :: struct {
    nick: string;
    email: string;
    online: bool;
    pending: [..] string;
}

Server :: struct {
    options: Server_Options;
    model: Model;
    listener: Io.Tcp_Listener;
    peers: [256] *Peer;
    users: [..] *User;
    group: Io.Wait_Group;
    stopping: bool;
    messages: s64;
    connections: s64;
    tls: Tls.Tls_Config;
    secure: bool;
    mail_group: Thread_Group;
    mail_inflight: s64;
}

Peer :: struct {
    server: *Server;
    slot: s64;
    link: Link;
    input: Io.Fiber_Handle;
    output: *Io.Channel(string);
    writer_done: Io.Wait_Group;
    user: *User;
    room: *Room;
    limiter: Limiter;
    closing: bool;
}

user_find :: (server: *Server, nick: string) -> *User {
    for server.users if it.nick == nick return it;
    return null;
}

queue :: (peer: *Peer, line: string) {
    if peer.closing return;

    owned := copy_string(line);
    if !Io.try_send(peer.output, owned) {
        free(owned);
        peer.closing = true;
        Io.cancel(peer.input);
    }
}

broadcast :: (server: *Server, room: *Room, line: string) {
    for server.peers {
        if it && it.room == room queue(it, line);
    }
}

leave :: (peer: *Peer) {
    if !peer.room return;
    room := peer.room;
    peer.room = null;
    broadcast(peer.server, room, tprint("LEFT % %", room.name, peer.user.nick));
}

write_peer :: (peer: *Peer) {
    defer Io.done(*peer.writer_done);

    while true {
        line, error := Io.recv(peer.output);
        if error != .NONE break;

        written := false;

        Io.with_deadline(2000, #code {
            written = send_line(*peer.link, line);
        });
        free(line);

        if !written {
            peer.closing = true;
            Io.cancel(peer.input);
            break;
        }
    }
}

execute :: (peer: *Peer, request: Request) {
    server := peer.server;
    if !allow(*peer.limiter, seconds_since_init(), server.options.rate) {
        queue(peer, "ERR rate-limit");
        return;
    }

    verb := request.verb;
    if verb == "QUIT" {
        queue(peer, "OK QUIT");
        peer.closing = true;
        return;
    }

    if verb == "PING" {
        queue(peer, "OK PING");
        return;
    }

    if verb == "NICK" {
        if peer.user {
            queue(peer, "ERR nickname-already-set");
            return;
        }

        user := user_find(server, request.target);
        if user && user.online {
            queue(peer, "ERR nickname-in-use");
            return;
        }

        if !user {
            if server.users.count >= MAX_USERS {
                queue(peer, "ERR user-limit");
                return;
            }

            user = New(User);
            user.nick = copy_string(request.target);
            array_add(*server.users, user);
        }

        user.online = true;
        peer.user = user;
        queue(peer, "OK NICK");

        return;
    }

    if !peer.user {
        queue(peer, "ERR set-nickname-first");
        return;
    }

    if verb == "EMAIL" {
        free(peer.user.email);
        peer.user.email = copy_string(request.target);
    } else if verb == "JOIN" {
        room := room_find(*server.model, request.target);
        if !room {
            queue(peer, "ERR room-limit");
            return;
        }

        leave(peer);
        peer.room = room;
        broadcast(server, room, tprint("JOINED % %", room.name, peer.user.nick));
    } else if verb == "LEAVE" {
        leave(peer);
    } else if verb == "DM" {
        recipient: *Peer;
        for server.peers if it && it.user && it.user.nick == request.target recipient = it;

        if !recipient {
            queue(peer, "ERR user-offline");
            return;
        }

        queue(recipient, tprint("PRIVATE % %", peer.user.nick, request.text));
    } else if verb == "HISTORY" {
        if !peer.room {
            queue(peer, "ERR join-first");
            return;
        }

        for peer.room.history queue(peer, tprint("HISTORY % % %", peer.room.name, it.nick, it.text));
    } else if verb == "SAY" {
        if !peer.room {
            queue(peer, "ERR join-first");
            return;
        }

        if !record_message(*server.model, peer.room, peer.user.nick, request.text) {
            queue(peer, "ERR persistence-failed");
            return;
        }

        server.messages += 1;
        broadcast(server, peer.room, tprint("MSG % % %", peer.room.name, peer.user.nick, request.text));

        for server.users {
            if !it.online && it.email.count > 0 && it.pending.count < 32 && mentions(request.text, it.nick) {
                array_add(*it.pending, copy_string(tprint("[%] %: %\r\n", peer.room.name, peer.user.nick, request.text)));
            }
        }
    }

    queue(peer, tprint("OK %", verb));
}

read_peer :: (peer: *Peer) {
    server := peer.server;

    peer.input = Io.current();

    defer {
        leave(peer);
        if peer.user peer.user.online = false;

        server.peers[peer.slot] = null;
        server.connections -= 1;
        Io.close(peer.output);
        Io.with_grace(-1, #code {
            Io.wait(*peer.writer_done);
        });

        while true {
            remaining, found := Io.try_recv(peer.output);
            if !found break;
            free(remaining);
        }

        Io.free_channel(peer.output);
        link_close(*peer.link);
        Io.done(*server.group);
        free(peer);
    }

    if server.secure {
        error: Tls.Tls_Error;

        Io.with_deadline(5000, #code {
            peer.link.tls, error = Tls.tls_accept(*server.tls, Io.reader_of(peer.link.tcp), Io.writer_of(peer.link.tcp));
        });

        if error.kind != .NONE {
            print("chat TLS accept: %\n", Tls.error_text(error));
            Io.close(peer.link.tcp);
            return;
        }
    }

    link_prepare(*peer.link);
    Io.add(*peer.writer_done);
    Io.spawn(write_peer, peer);
    queue(peer, "READY chat/1");

    while !peer.closing {
        line, error := Io.read_line(*peer.link.reader);
        if error != .NONE || peer.link.reader.unterminated break;

        request, valid := parse_request(line);
        if valid execute(peer, request);
        else queue(peer, "ERR syntax");

        reset_temporary_storage();
    }
}

monitor :: (server: *Server) {
    defer Io.done(*server.group);
    previous := 0;

    while !server.stopping {
        if Io.interrupted() {
            server.stopping = true;
            Io.close(server.listener);
            for server.peers if it Io.cancel(it.input);
            break;
        }

        publish_stats(server, server.messages - previous);
        previous = server.messages;
        send_digests(server);
        Io.sleep(1000);
        reset_temporary_storage();
    }
}

run_server :: (options: Server_Options) {
    server: Server;
    server.options = options;
    server.model.history_limit = options.history;
    defer model_free(*server.model);

    if !model_open(*server.model, options.file) {
        print("chat: cannot recover log\n");
        exit(1);
    }

    if options.tls_cert.count > 0 || options.tls_key.count > 0 {
        cert, cert_ok := read_entire_file(options.tls_cert);
        key, key_ok := read_entire_file(options.tls_key);

        defer free(cert);
        defer free(key);

        if !cert_ok || !key_ok {
            print("chat: certificate and key required\n");
            exit(1);
        }

        server.tls.certificate_chain_pem = cert;
        server.tls.private_key_pem = key;
        error := Tls.tls_config_init(*server.tls);
        if error.kind != .NONE {
            print("TLS: %\n", Tls.error_text(error));
            exit(1);
        }

        server.secure = true;
    }

    defer if server.secure Tls.tls_config_free(*server.tls);
    init(*server.mail_group, 1, deliver_mail, temporary_storage_size = 1 << 20);
    server.mail_group.logging = false;
    start(*server.mail_group);
    error: Io.Io_Error;
    server.listener, error = Io.tcp_listen(options.listen);
    if error != .NONE {
        print("chat: listen failed: %\n", error);
        shutdown(*server.mail_group);
        exit(1);
    }

    address, _ := Io.local_address(server.listener);

    print("LISTEN %\n", Io.address_to_string(address));
    Io.add(*server.group);
    Io.spawn(monitor, *server);

    while !server.stopping {
        tcp, accept_error := Io.accept(server.listener);
        if accept_error != .NONE break;

        slot := -1;

        for server.peers if !it {
            slot = it_index;
            break;
        }

        if slot < 0 {
            Io.close(tcp);
            continue;
        }

        peer := New(Peer);
        peer.server = *server;
        peer.slot = slot;
        peer.link.tcp = tcp;
        peer.output = Io.make_channel(string, 128);
        server.peers[slot] = peer;
        server.connections += 1;
        Io.add(*server.group);
        peer.input = Io.spawn(read_peer, peer);
    }

    server.stopping = true;
    for server.peers if it Io.cancel(it.input);

    Io.wait(*server.group);
    send_digests(*server);
    wait_for_all(*server.mail_group);
    shutdown(*server.mail_group);

    for server.users {
        free(it.nick);
        free(it.email);
        for pending: it.pending free(pending);

        array_free(it.pending);
        free(it);
    }

    array_free(server.users);
    print("STOPPED messages=%\n", server.messages);
}
