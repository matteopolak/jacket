// What a fiber that overflows its stack runs into: the guard pages at the low end of every stack
// (stacks.jai). Touching one is a SIGSEGV or SIGBUS, which the handler here recognises by the
// address and answers with a message that names the problem, instead of the bare "fatal runtime
// signal" the crash handler gives for any fault. A fault anywhere else is handed back to whatever
// handler was installed before. Windows reports the access violation through the runtime's own
// exception filter, so nothing is installed there.
//
// The handler must not run on the overflowing stack, so each scheduler thread gets an alternate
// signal stack. Its first word is the thread's Scheduler, which is how the handler, knowing
// nothing else, finds the running fiber and its guard.

#if OS == .WINDOWS {
    guard_overflow :: (scheduler: *Scheduler) {}

    unguard_overflow :: (scheduler: *Scheduler) {}
} else {
    POSIX :: #import "POSIX";

    ALT_STACK_BYTES :: 64 * 1024;

    SIGNAL_SEGV :: 11;

    #if OS == .MACOS {
        SIGNAL_BUS :: 10;
        FLAG_SIGINFO :: 0x40;
        FLAG_ONSTACK :: 0x1;
    } else {
        SIGNAL_BUS :: 7;
        FLAG_SIGINFO :: 0x4;
        FLAG_ONSTACK :: 0x08000000;
    }

    previous_actions: [2] POSIX.sigaction_t;
    handler_installed: s32;

    guard_overflow :: (scheduler: *Scheduler) {
        if compare_and_swap(*handler_installed, 0, 1) {
            action: POSIX.sigaction_t;
            action.sa_sigaction = overflow_handler;
            action.sa_flags = FLAG_SIGINFO | FLAG_ONSTACK;
            POSIX.sigemptyset(*action.sa_mask);

            POSIX.sigaction(SIGNAL_SEGV, *action, *previous_actions[0]);
            POSIX.sigaction(SIGNAL_BUS, *action, *previous_actions[1]);
        }

        scheduler.alt_memory = alloc(ALT_STACK_BYTES);
        (cast(**Scheduler) scheduler.alt_memory).* = scheduler;

        stack: POSIX.stack_t;
        stack.ss_sp = scheduler.alt_memory;
        stack.ss_size = ALT_STACK_BYTES;
        POSIX.sigaltstack(*stack, *scheduler.alt_previous);
    }

    unguard_overflow :: (scheduler: *Scheduler) {
        POSIX.sigaltstack(*scheduler.alt_previous, null);
        free(scheduler.alt_memory);
        scheduler.alt_memory = null;
    }

    fault_address :: (info: *POSIX.siginfo_t) -> u64 #c_call {
        #if OS == .MACOS return cast(u64) info.si_addr;
        else return cast(u64) info._sifields._sigfault.si_addr;
    }

    overflow_handler :: (signal: s32, info: *POSIX.siginfo_t, secret: *void) #c_call {
        alternate: POSIX.stack_t;

        POSIX.sigaltstack(null, *alternate);

        scheduler := (cast(**Scheduler) alternate.ss_sp).*;
        if scheduler && scheduler.current {
            fiber := scheduler.current;
            address := fault_address(info);
            low := cast(u64) fiber.stack.base;

            // A little below the guard too: a frame can start past its first page.
            if address + 4096 >= low && address < low + cast(u64) fiber.stack.guard {
                write_text("fatal: fiber stack overflow: fiber ");
                write_number(fiber.id);
                write_text(" used up its ");
                write_number(cast(u64) (fiber.stack.size - fiber.stack.guard));
                write_text(" byte stack (the STACK_BYTES parameter of the Extensions/Io import sets it)\n");
                POSIX._exit(139);
            }
        }

        // Not a fiber's guard: let the fault happen again under the handler that was there before.
        which := ifx signal == SIGNAL_SEGV then 0 else 1;

        POSIX.sigaction(signal, *previous_actions[which], null);
    }

    write_text :: (text: string) #c_call {
        POSIX.write(2, text.data, cast(u64) text.count);
    }

    write_number :: (number: u64) #c_call {
        digits: [20] u8;
        position := digits.count;
        value := number;

        while true {
            position -= 1;
            digits[position] = cast(u8) (#char "0" + value % 10);
            value /= 10;
            if value == 0 break;
        }

        POSIX.write(2, digits.data + position, cast(u64) (digits.count - position));
    }
}
