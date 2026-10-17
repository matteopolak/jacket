// Independent real SDK failures; verification runs with the fatal handler.
// Reviewed thread.md lifecycle plus SDK ownership: failed destruction retains it.
#import "Basic"()(ENABLE_ASSERT = true);
T :: #import "Thread";

errors: s64;
assertions: s64;
#if !#exists(EXPECTED_THREAD_ASSERTIONS) EXPECTED_THREAD_ASSERTIONS :: 1;

capture_log :: (message: string, data: *void, info: Log_Info) {
    if (info.common_flags & .ERROR) != 0 errors += 1;
}

continue_assertion :: (location: Source_Code_Location, message: string) -> bool {
    assertions += 1;
    return false;
}

main :: () {
    previous_log := context.logger;
    previous_assert := context.assertion_failed;
    semaphore: T.Semaphore;

    T.init(*semaphore);

    #if OS == .MACOS {
        owner := semaphore.owner;
        event := semaphore.event;

        semaphore.owner = 0; // Invalid task right: actual Mach call must fail.
        context.logger = capture_log;
        context.assertion_failed = continue_assertion;
        T.destroy(*semaphore);
        context.logger = previous_log;
        context.assertion_failed = previous_assert;
        assert(errors == 1 && semaphore.event == event);
        assert(assertions == EXPECTED_THREAD_ASSERTIONS);
        semaphore.owner = owner;
    }

    T.signal(*semaphore);
    status := T.wait_for(*semaphore, 0);

    assert(status == .SUCCESS);
    T.destroy(*semaphore);
    print("ok\n");
}
