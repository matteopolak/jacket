// Channels between fibers (unbuffered and buffered), wait groups and fiber mutexes.
#import "Basic";
Io :: #import "Extensions/Io";

Ping :: struct {
    number: int;
    label: string;
}

ping_pong_rounds: int;

// Two fibers pass a counter back and forth over unbuffered channels.
player :: (ins: *Io.Channel(int), outs: *Io.Channel(int)) {
    while true {
        value, error := Io.recv(ins);
        if error != .NONE break;

        ping_pong_rounds += 1;
        if Io.send(outs, value + 1) != .NONE break;
    }
}

Players :: struct {
    ins: *Io.Channel(int);
    outs: *Io.Channel(int);
}

run_player :: (players: Players) {
    player(players.ins, players.outs);
}

test_ping_pong :: () {
    to_b := Io.make_channel(int);
    to_a := Io.make_channel(int);

    Io.spawn(run_player, Players.{ to_b, to_a });

    value := 0;

    for 1..1000 {
        assert(Io.send(to_b, value) == .NONE);
        reply, error := Io.recv(to_a);

        assert(error == .NONE);
        assert(reply == value + 1, "reply % after %", reply, value);
        value = reply;
    }

    assert(value == 1000);
    assert(ping_pong_rounds == 1000);

    // Closing the channel ends the player's loop; its pending recv returns .CLOSED.
    Io.close(to_b);
    Io.yield();

    assert(Io.send(to_b, 5) == .CLOSED);
    _, error := Io.recv(to_b);

    assert(error == .CLOSED);

    Io.free_channel(to_b);
    Io.free_channel(to_a);
}

// A buffered channel decouples a producer from a consumer and keeps the order of values.
producer_sent: int;

produce :: (channel: *Io.Channel(Ping)) {
    for 1..100 {
        error := Io.send(channel, Ping.{ it, "p" });
        assert(error == .NONE);
        producer_sent += 1;
    }

    Io.close(channel);
}

test_buffered :: () {
    channel := Io.make_channel(Ping, 8);

    Io.spawn(produce, channel);

    expected := 1;

    while true {
        ping, error := Io.recv(channel);
        if error == .CLOSED break;

        assert(error == .NONE);
        assert(ping.number == expected && ping.label == "p", "got % expected %", ping.number, expected);
        expected += 1;
    }

    assert(expected == 101);
    assert(producer_sent == 100);

    // try_send and try_recv never block.
    full := Io.make_channel(int, 2);

    assert(Io.try_send(full, 1));
    assert(Io.try_send(full, 2));
    assert(!Io.try_send(full, 3));

    first, found := Io.try_recv(full);

    assert(found && first == 1);
    assert(Io.try_send(full, 3));

    second, third: int;
    second, found = Io.try_recv(full);
    assert(found && second == 2);
    third, found = Io.try_recv(full);
    assert(found && third == 3);
    _, found = Io.try_recv(full);
    assert(!found);

    // Closing keeps what is buffered.
    Io.try_send(full, 9);
    Io.close(full);
    value, error := Io.recv(full);

    assert(error == .NONE && value == 9);
    _, error = Io.recv(full);
    assert(error == .CLOSED);

    Io.free_channel(full);
    Io.free_channel(channel);
}

// Many senders block on a small buffer; one receiver takes everything exactly once.
test_fan_in :: () {
    SENDERS :: 50;
    PER_SENDER :: 20;

    channel := Io.make_channel(int, 4);
    group: Io.Wait_Group;

    Io.add(*group, SENDERS);

    for 0..SENDERS - 1 {
        Io.spawn(fan_in_sender, Fan_In.{ channel, *group, it });
    }

    seen: [SENDERS * PER_SENDER] bool;

    for 1..SENDERS * PER_SENDER {
        value, error := Io.recv(channel);
        assert(error == .NONE);
        assert(!seen[value], "value % arrived twice", value);
        seen[value] = true;
    }

    Io.wait(*group);
    for seen assert(it);

    assert(Io.try_recv(channel) == 0, "nothing is left");

    Io.free_channel(channel);
}

Fan_In :: struct {
    channel: *Io.Channel(int);
    group: *Io.Wait_Group;
    index: int;
}

fan_in_sender :: (job: Fan_In) {
    for 0..19 Io.send(job.channel, job.index * 20 + it);
    Io.done(job.group);
}

// A Fiber_Mutex can be held across a blocking call; waiters get it in order.
mutex: Io.Fiber_Mutex;
critical_depth: int;
max_critical_depth: int;
lock_order: [..] int;

locker :: (index: int) {
    assert(Io.lock(*mutex) == .NONE);

    critical_depth += 1;
    max_critical_depth = max(max_critical_depth, critical_depth);
    array_add(*lock_order, index);
    Io.sleep(1);
    critical_depth -= 1;

    Io.unlock(*mutex);
    Io.done(*mutex_group);
}

mutex_group: Io.Wait_Group;

test_mutex :: () {
    Io.add(*mutex_group, 20);
    for 0..19 Io.spawn(locker, it);

    Io.wait(*mutex_group);
    assert(max_critical_depth == 1, "two fibers held the mutex at once");
    assert(lock_order.count == 20);
    for lock_order assert(it == it_index, "waiters must get the lock in order: % at %", it, it_index);

    assert(Io.try_lock(*mutex));
    assert(!Io.try_lock(*mutex));
    Io.unlock(*mutex);
}

// A wait group that is already at zero does not block, and one fiber can wait while several finish.
test_wait_group :: () {
    group: Io.Wait_Group;

    assert(Io.wait(*group) == .NONE);

    finished: [3] bool;

    Io.add(*group, 3);
    for 0..2 Io.spawn(group_member, Member.{ *group, it, *finished });

    assert(!finished[0]);

    Io.wait(*group);
    for finished assert(it);
}

Member :: struct {
    group: *Io.Wait_Group;
    index: int;
    finished: *[3] bool;
}

group_member :: (member: Member) {
    Io.sleep(5 * (member.index + 1));
    member.finished.*[member.index] = true;
    Io.done(member.group);
}

root :: () {
    test_ping_pong();
    test_buffered();
    test_fan_in();
    test_mutex();
    test_wait_group();
}

main :: () {
    Io.io_run(root);
    print("ok\n");
}
