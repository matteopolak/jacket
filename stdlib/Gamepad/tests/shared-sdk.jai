// test: skip_on(.WASM, reason = "needs the native SDL library, which has no wasm build") skip_on(.WINDOWS, cpu = .ARM64)
#import "Basic";
G :: #import "Gamepad";
SDL :: #import "SDL";

main :: () {
    assert(SDL.SDL_InitSubSystem(SDL.SDL_INIT_EVENTS) == 0);
    event: SDL.SDL_Event;
    event.type = SDL.SDL_USEREVENT;
    assert(SDL.SDL_PushEvent(*event) == 1);
    before := SDL.SDL_GameControllerEventState(-1);

    G.init_gamepad();
    assert(SDL.SDL_HasEvent(cast(u32) SDL.SDL_USEREVENT) != .SDL_FALSE, "gamepad initialization discarded a caller event");
    assert(SDL.SDL_GameControllerEventState(-1) == before, "gamepad initialization changed a caller event policy");
    print("ok\n");
}
