// `#must` on one of several results makes discarding the call's results an error.
// rules: note.6
pick :: () -> int, ok: bool #must {
    return 1, true;
}

main :: () {
    pick();
}
