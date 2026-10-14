#import "Basic";
#import "Extensions/SDL3";

calls: s32;
state: s32 = 314;

run_app :: (argc: s32, argv: **u8) -> s32 #c_call {
    calls += 1;
    return 73;
}

app_init :: (appstate: **void, argc: s32, argv: **u8) -> SDL_AppResult #c_call {
    calls += 1;
    appstate.* = *state;
    return SDL_APP_SUCCESS;
}

app_iterate :: (appstate: *void) -> SDL_AppResult #c_call {
    push_context, defer_pop;
    assert(false);
    return SDL_APP_FAILURE;
}

app_event :: (appstate: *void, event: *SDL_Event) -> SDL_AppResult #c_call {
    push_context, defer_pop;
    assert(false);
    return SDL_APP_FAILURE;
}

app_quit :: (appstate: *void, result: SDL_AppResult) #c_call {
    push_context, defer_pop;
    assert(appstate == *state);
    assert(result == SDL_APP_SUCCESS);
    calls += 1;
}

main :: () {
    SDL_SetMainReady();
    assert(SDL_RunApp(0, null, run_app, null) == 73);
    assert(calls == 1);
    calls = 0;
    assert(SDL_EnterAppMainCallbacks(0, null, app_init, app_iterate, app_event, app_quit) == 0);
    assert(calls == 2);
    print("SDL3 main and application callbacks passed\n");
}
