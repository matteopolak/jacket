// Independent dependency regression; never treated as a passing audio gate.
#import "Basic"(ENABLE_ASSERT = false);
#import "Thread";

main :: () {
    semaphore: Semaphore;

    init(*semaphore);
    signal(*semaphore);
    result := wait_for(*semaphore, 0);

    print("semaphore-after-init-signal: %\n", result);
    if result != .SUCCESS exit(1);

    destroy(*semaphore);
}
