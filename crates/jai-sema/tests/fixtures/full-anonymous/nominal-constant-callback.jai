Event :: struct { value: s32; }
Listener :: struct { callback: (event: *Event) -> s32 #c_call; }
listener :: Listener.{ callback = (event: *Event) -> s32 #c_call {
    return event.value + 2;
} };
main :: () -> int {
    event: Event = .{ value = 40 };
    return listener.callback(*event);
}
