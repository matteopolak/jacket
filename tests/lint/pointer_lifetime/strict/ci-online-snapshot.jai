#import "Basic";

Entry :: struct {
    player: string;
}

Snapshot :: struct {
    status: string;
    board: [] Entry;
}

online_snapshot :: (enabled: bool = true) -> Snapshot {
    snapshot: Snapshot;

    if !enabled {
        snapshot.status = "offline";
        return snapshot;
    }

    snapshot.status = copy_string("online",, temp);
    snapshot.board.count = 1;
    snapshot.board.data = alloc(size_of(Entry),, temp);
    snapshot.board[0].player = copy_string("hello",, temp);

    return snapshot;
}

main :: () {
    value := online_snapshot();
    assert(value.board[0].player == "hello");
    reset_temporary_storage();
}

saved: Snapshot;

keep :: () {
    saved = online_snapshot();
}

read_after_reset :: () {
    value := online_snapshot();
    reset_temporary_storage();
    assert(value.board[0].player == "hello");
}
