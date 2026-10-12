// SIGINT and SIGTERM as a flag a server polls, so Ctrl-C can end a program cleanly instead of
// killing it in the middle of a response. POSIX only: Windows has no signals to catch here, and
// `interrupted` stays false.
//
//     catch_interrupts();
//     while !interrupted() sleep(100);

#scope_export

// Makes SIGINT and SIGTERM set the flag `interrupted` reads, instead of ending the process. The
// handler is installed once for the whole program; calling this again does nothing.
catch_interrupts :: () {
    #if OS != .WINDOWS {
        if compare_and_swap(*interrupt_installed, 0, 1) {
            action: POSIX.sigaction_t;
            action.sa_sigaction = interrupt_handler;
            action.sa_flags = FLAG_SIGINFO;
            POSIX.sigemptyset(*action.sa_mask);

            POSIX.sigaction(SIGNAL_INT, *action, null);
            POSIX.sigaction(SIGNAL_TERM, *action, null);
        }
    }
}

// True once SIGINT or SIGTERM arrived after `catch_interrupts`, or `request_interrupt` ran.
interrupted :: () -> bool {
    return atomic_load(*interrupt_flag) != 0;
}

// Sets the flag by hand, as a signal would (tests and programs that stop themselves).
request_interrupt :: () {
    atomic_store(*interrupt_flag, 1);
}

// Clears the flag (tests that run several servers in one process).
clear_interrupt :: () {
    atomic_store(*interrupt_flag, 0);
}

#scope_module

interrupt_flag: s32;
interrupt_installed: s32;

SIGNAL_INT :: 2;
SIGNAL_TERM :: 15;

#if OS != .WINDOWS {
    interrupt_handler :: (signal: s32, info: *POSIX.siginfo_t, secret: *void) #c_call {
        // A plain store: the handler has no context, and a word-sized store is atomic enough.
        interrupt_flag = 1;
    }
}
