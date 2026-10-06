// Nothing uses this struct, but it is still type-checked, so `Kind` must exist by then.
Kind_Table :: struct {
    spawn: (kind: Kind) -> int;
    count: int;
}

label_of :: (kind: Kind) -> string { return Kind_Labels[kind]; }
