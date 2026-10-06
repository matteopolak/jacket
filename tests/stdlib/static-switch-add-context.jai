// A module that loads its `#add_context` file from a top-level `#if OS == { case ...; }`: the
// switch is expanded before any compile-time code lays out the Context, and evaluating `OS`
// there does not expand the module's other pending items (its `using settings: Settings`).
#import "Basic";
#import "Switch_Context";

MODE_COUNT :: #run enum_names(Mode).count;
Mode :: enum {
    A;
    B;
    C;
}

main :: () {
    state: Backend_State;
    state.frames = 3;
    context.backend_state = *state;
    assert(MODE_COUNT == 3);
    assert(backend_ready());
    print("ok\n");
}
