// SDL2 without a window: the library version and platform name, the rectangle and line geometry
// helpers, hints, the timer subsystem and its clocks, and the error string. SDL2 is the installed
// library on macOS and Linux (Homebrew's `sdl2`, the distribution's libsdl2) and the official
// release's import library and DLL on Windows (tools/native-libs.json), so this checks the module
// against whichever copy the toolchain finds.
#import "Basic";
#import "SDL";

main :: () {
    version: SDL_version;
    SDL_GetVersion(*version);
    assert(version.major == 2, "SDL version %.%.%", version.major, version.minor, version.patch);

    platform := to_string(SDL_GetPlatform());
    expected := ifx OS == .WINDOWS then "Windows" else ifx OS == .MACOS then "Mac OS X" else "Linux";
    assert(platform == expected, "platform '%', expected '%'", platform, expected);

    // Rectangles: two overlapping boxes, one far away.
    a := SDL_Rect.{ 0, 0, 10, 10 };
    b := SDL_Rect.{ 5, 5, 10, 10 };
    far := SDL_Rect.{ 100, 100, 4, 4 };
    assert(SDL_HasIntersection(*a, *b) == .SDL_TRUE, "a and b overlap");
    assert(SDL_HasIntersection(*a, *far) == .SDL_FALSE, "a and far do not");
    overlap: SDL_Rect;
    assert(SDL_IntersectRect(*a, *b, *overlap) == .SDL_TRUE);
    assert(overlap.x == 5 && overlap.y == 5 && overlap.w == 5 && overlap.h == 5, "intersection %", overlap);
    united: SDL_Rect;
    SDL_UnionRect(*a, *b, *united);
    assert(united.x == 0 && united.y == 0 && united.w == 15 && united.h == 15, "union %", united);

    points := SDL_Point.[.{ 3, 4 }, .{ 10, -2 }, .{ -5, 8 }];
    box: SDL_Rect;
    assert(SDL_EnclosePoints(points.data, cast(s32) points.count, null, *box) == .SDL_TRUE, "enclose");
    assert(box.x == -5 && box.y == -2 && box.w == 16 && box.h == 11, "enclosing box %", box);

    // A line through the box is clipped to its edges; one beside it is rejected.
    x1, y1, x2, y2: s32 = -5, 5, 20, 5;
    assert(SDL_IntersectRectAndLine(*a, *x1, *y1, *x2, *y2) == .SDL_TRUE, "line crosses the rectangle");
    assert(x1 == 0 && x2 == 9 && y1 == 5 && y2 == 5, "clipped line (%, %) to (%, %)", x1, y1, x2, y2);
    p1, q1, p2, q2: s32 = -5, 50, 20, 50;
    assert(SDL_IntersectRectAndLine(*a, *p1, *q1, *p2, *q2) == .SDL_FALSE, "line beside the rectangle");

    // Hints round-trip through the library's table.
    assert(SDL_SetHint("SDL_JAI_TEST_HINT", "on") == .SDL_TRUE, "set hint");
    assert(to_string(SDL_GetHint("SDL_JAI_TEST_HINT")) == "on", "hint reads back");

    // The timer subsystem needs no display. Delays and clocks agree to within scheduling noise.
    assert(SDL_Init(.SDL_INIT_TIMER) == 0, "SDL_Init: %", to_string(SDL_GetError()));
    assert((SDL_WasInit(.SDL_INIT_TIMER) & .SDL_INIT_TIMER) != 0, "timer subsystem is up");
    started := SDL_GetTicks();
    counter := SDL_GetPerformanceCounter();
    SDL_Delay(30);
    elapsed := SDL_GetTicks() - started;
    ticks := cast(float64) (SDL_GetPerformanceCounter() - counter) / cast(float64) SDL_GetPerformanceFrequency();
    assert(elapsed >= 25 && elapsed < 5000, "SDL_Delay(30) took % ms", elapsed);
    assert(ticks > 0.025 && ticks < 5, "the performance counter measured % s", ticks);
    SDL_Quit();
    assert(SDL_WasInit(.SDL_INIT_EVERYTHING) == 0, "SDL_Quit shuts everything down");
    print("sdl2 ok\n");
}
