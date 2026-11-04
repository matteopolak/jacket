#import "Basic";
T :: #import "Thread";

worker :: (thread: *T.Thread) -> s64 {
    return thread.index;
}

main :: () {
    threads: [3] T.Thread;

    for index: 0..2 {
        T.thread_init(*threads[index], worker);
        T.thread_start(*threads[index]);
    }

    for index: 0..2 {
        assert(T.thread_is_done(*threads[index], -1));
        T.thread_deinit(*threads[index]);
    }
}

iterated :: () {
    threads: [3] T.Thread;
    for *threads {
        T.thread_init(it, worker);
        T.thread_start(it);
    }

    for *threads {
        assert(T.thread_is_done(it, -1));
        T.thread_deinit(it);
    }
}

unfinished :: () {
    thread: T.Thread;
    T.thread_init(*thread, worker);
    T.thread_start(*thread);
}

wrong_receiver :: () {
    threads: [2] T.Thread;
    T.thread_init(*threads[0], worker);
    T.thread_start(*threads[0]);
    T.thread_deinit(*threads[1]);
}
