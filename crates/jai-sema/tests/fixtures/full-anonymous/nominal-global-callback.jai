Request :: struct { value: s32; }
Reply :: struct { value: s32; }
Listener :: struct { callback: (request: Request) -> Reply #c_call; }
offset: s32 = 2;
listeners: [1] Listener = .[
    Listener.{ callback = (request: Request) -> Reply #c_call {
        return .{ value = request.value + offset };
    } }
];
main :: () -> int {
    reply := listeners[0].callback(Request.{ value = 40 });
    return reply.value;
}
