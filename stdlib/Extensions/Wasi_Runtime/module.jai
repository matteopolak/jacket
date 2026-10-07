// Wasi_Runtime: what a wasm64 program needs to run as a WASI command (node, wasmtime and the
// like). jaic adds it to `jaic build -os wasm` programs and to workspaces whose LLVM triple
// names WASI; a program that brings its own host bindings leaves it out. This module is a jaic
// extension, not official Jai. See docs/native/wasm-target.md.
//
// It supplies, in Jai, the C library entry points the stdlib reaches for under `OS == .WASM`
// (the same ones the interpreter's sandbox host implements): a heap, `memmove`/`memcmp`,
// `read`/`write`, `clock_gettime`, `exit`, files and directories (files.jai), process calls that
// fail the way a sandbox without processes would, a libm (math.jai, elementary.jai), the
// compiler-rt binary128 helpers Long_Double needs (quad.jai), plus Runtime_Support's
// `wasm_write_string` and `wasm_debug_break`, and `_start`, which collects the WASI arguments and
// calls `main`. Every definition is a `#program_export` under its C name; jaic's LLVM backend
// binds the program's `#foreign` references to them without exporting them from the module, so
// the linker drops the ones a program never calls and the module imports nothing but the
// `wasi_snapshot_preview1` calls it uses.
//
// Under the interpreter (compile-time code, `jaic run -os wasm`) none of this runs: the
// sandbox host answers those `#foreign` calls itself.

#assert OS == .WASM "Wasi_Runtime is only for wasm targets";

#scope_module

#load "files.jai";
#load "math.jai";
#load "elementary.jai";
#load "quad.jai";

c_runtime :: #system_library "libc";
wasi :: #system_library "wasi_snapshot_preview1";

// Runtime_Support's entry point, `#program_export "main"`.
program_main :: (argc: s32, argv: **u8) -> s32 #foreign c_runtime "main";

// End of the linker's data: the heap starts here.
__heap_base: u8 #elsewhere c_runtime;

memory_page_count :: (memory: s32) -> s64 #intrinsic "llvm.wasm.memory.size.i64";
memory_add_pages :: (memory: s32, pages: s64) -> s64 #intrinsic "llvm.wasm.memory.grow.i64";
trap :: () #intrinsic "llvm.trap";

// WASI preview 1 is specified for 32-bit memories, so every address it takes or fills in is a
// 32-bit offset. The stack and the heap start at the bottom of memory, so ours fit until the
// program has allocated 4 GiB.
wasi_fd_write :: (fd: s32, iovecs: u32, iovec_count: u32, written: u32) -> s32 #foreign wasi "fd_write";
wasi_fd_read :: (fd: s32, iovecs: u32, iovec_count: u32, read: u32) -> s32 #foreign wasi "fd_read";
wasi_args_sizes_get :: (count: u32, text_bytes: u32) -> s32 #foreign wasi "args_sizes_get";
wasi_args_get :: (pointers: u32, text: u32) -> s32 #foreign wasi "args_get";
wasi_environ_sizes_get :: (count: u32, text_bytes: u32) -> s32 #foreign wasi "environ_sizes_get";
wasi_environ_get :: (pointers: u32, text: u32) -> s32 #foreign wasi "environ_get";
wasi_clock_time_get :: (clock: u32, precision: u64, nanoseconds: u32) -> s32 #foreign wasi "clock_time_get";
wasi_proc_exit :: (code: s32) #foreign wasi "proc_exit";

// One WASI `ciovec`: a 32-bit address and length.
Span32 :: struct {
    address: u32;
    length: u32;
}

offset32 :: (p: *void) -> u32 #no_context {
    return cast,no_check(u32) cast(u64) p;
}

widen :: (offset: u32) -> *u8 #no_context {
    return cast(*u8) cast(u64) offset;
}

// ---------------------------------------------------------------------------------------------
// Entry point

#program_export "_start"
wasi_start :: () #c_call {
    count, text_bytes: u32;
    if wasi_args_sizes_get(offset32(*count), offset32(*text_bytes)) != 0 {
        count = 0;
        text_bytes = 0;
    }
    // WASI hands out 32-bit pointers; Jai's argv holds 64-bit ones, so read into a table of
    // offsets and widen it in place from the end.
    table := cast(*u64) heap_take(cast(u64) (count + 1) * 8);
    text := heap_take(cast(u64) text_bytes + 1);
    if !table || !text wasi_proc_exit(71);
    if count > 0 && wasi_args_get(offset32(table), offset32(text)) == 0 {
        offsets := cast(*u32) table;
        for < i: 0..cast(s64) count - 1 table[i] = cast(u64) offsets[i];
    } else {
        count = 0;
    }
    table[count] = 0;
    wasi_proc_exit(program_main(cast(s32) count, cast(**u8) table));
}

// ---------------------------------------------------------------------------------------------
// Output, input and process

#program_export "wasm_write_string"
wasi_write_string :: (count: s64, data: *void, to_standard_error: bool) #c_call {
    write_all(ifx to_standard_error then 2 else 1, data, count);
}

#program_export "wasm_debug_break"
wasi_debug_break :: () #c_call {
    message :: "debug_break() reached; stopping.\n";
    write_all(2, message.data, message.count);
    trap();
}

#program_export "write"
wasi_write :: (fd: s32, buffer: *void, count: u64) -> s64 #c_call {
    span := Span32.{ offset32(buffer), cast,no_check(u32) count };
    written: u32;
    if wasi_fd_write(fd, offset32(*span), 1, offset32(*written)) != 0 return -1;
    return written;
}

#program_export "read"
wasi_read :: (fd: s32, buffer: *void, count: u64) -> s64 #c_call {
    span := Span32.{ offset32(buffer), cast,no_check(u32) count };
    got: u32;
    if wasi_fd_read(fd, offset32(*span), 1, offset32(*got)) != 0 return -1;
    return got;
}

#program_export "exit"
wasi_exit :: (code: s32) #c_call {
    wasi_proc_exit(code);
}

#program_export "_exit"
wasi_exit_now :: (code: s32) #c_call {
    wasi_proc_exit(code);
}

#program_export "abort"
wasi_abort :: () #c_call {
    trap();
}

#program_export "isatty"
wasi_isatty :: (fd: s32) -> s32 #c_call {
    return 0;
}

// The environment is read once, on the first lookup, into heap memory that stays for the run.
#program_export "getenv"
wasi_getenv :: (name: *u8) -> *u8 #c_call {
    if !environment_ready {
        environment_ready = true;
        load_environment();
    }
    name_length := c_length(name);
    entry := environment;
    while entry && entry.* {
        candidate := entry.*;
        if c_length(candidate) > name_length && candidate[name_length] == #char "=" && bytes_equal(candidate, name, name_length) {
            return candidate + name_length + 1;
        }
        entry += 1;
    }
    return null;
}

// CLOCK_REALTIME and CLOCK_MONOTONIC have the same numbers in WASI as in Linux.
#program_export "clock_gettime"
wasi_clock_gettime :: (clock: s32, result: *Seconds_And_Nanoseconds) -> s32 #c_call {
    nanoseconds: u64;
    if wasi_clock_time_get(cast,no_check(u32) clock, 1, offset32(*nanoseconds)) != 0 return -1;
    result.seconds = cast(s64) (nanoseconds / 1_000_000_000);
    result.nanoseconds = cast(s64) (nanoseconds % 1_000_000_000);
    return 0;
}

Seconds_And_Nanoseconds :: struct {
    seconds: s64;
    nanoseconds: s64;
}

// Sleeping polls on relative CLOCK_MONOTONIC timeouts until the clock reaches the deadline: a
// host may wake a poll slightly early (Node's can). The whole interval is slept, so `remaining`
// is never written.
#program_export "nanosleep"
wasi_nanosleep :: (requested: *Seconds_And_Nanoseconds, remaining: *Seconds_And_Nanoseconds) -> s32 #c_call {
    if !requested || requested.seconds < 0 || requested.nanoseconds < 0 || requested.nanoseconds >= 1_000_000_000 {
        error_number = EINVAL;
        return -1;
    }
    total := cast(u64) requested.seconds * 1_000_000_000 + cast(u64) requested.nanoseconds;
    if total == 0 return 0;
    now: u64;
    if wasi_clock_time_get(1, 1, offset32(*now)) != 0 now = 0;
    deadline := now + total;
    while true {
        subscription: Clock_Subscription;
        subscription.kind = 0; // clock
        subscription.clock_id = 1; // monotonic
        subscription.timeout = total;
        event: Poll_Event;
        happened: u32;
        if wasi_poll_oneoff(offset32(*subscription), offset32(*event), 1, offset32(*happened)) != 0 {
            error_number = EINVAL;
            return -1;
        }
        if now == 0 || wasi_clock_time_get(1, 1, offset32(*now)) != 0 || now >= deadline break;
        total = deadline - now;
    }
    return 0;
}

// WASI has no time zones: local time is UTC, as in the interpreter's sandbox host.
#program_export "localtime_r"
wasi_localtime_r :: (time: *s64, result: *Broken_Down_Time) -> *Broken_Down_Time #c_call {
    return wasi_gmtime_r(time, result);
}

#program_export "gmtime_r"
wasi_gmtime_r :: (time: *s64, result: *Broken_Down_Time) -> *Broken_Down_Time #c_call {
    if !time || !result return null;
    seconds := time.*;
    days := seconds / 86400;
    second_of_day := seconds % 86400;
    if second_of_day < 0 {
        second_of_day += 86400;
        days -= 1;
    }
    // Shift to a calendar whose years start on March 1, so the leap day ends each year.
    shifted := days + 719468;
    cycles := shifted / 146097;
    if shifted < 0 && shifted % 146097 != 0 cycles -= 1;
    day_of_cycle := shifted - cycles * 146097;
    year_of_cycle := (day_of_cycle - day_of_cycle / 1460 + day_of_cycle / 36524 - day_of_cycle / 146096) / 365;
    day_of_year := day_of_cycle - (365 * year_of_cycle + year_of_cycle / 4 - year_of_cycle / 100);
    month_from_march := (5 * day_of_year + 2) / 153;
    day := day_of_year - (153 * month_from_march + 2) / 5 + 1;
    month := ifx month_from_march < 10 then month_from_march + 3 else month_from_march - 9;
    year := year_of_cycle + cycles * 400;
    if month <= 2 year += 1;
    leap := (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    days_before_month :: s64.[0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    day_in_year := days_before_month[month - 1] + day - 1;
    if leap && month > 2 day_in_year += 1;
    weekday := (days + 4) % 7; // 1970-01-01 was a Thursday.
    if weekday < 0 weekday += 7;
    result.* = .{};
    result.second = cast(s32) (second_of_day % 60);
    result.minute = cast(s32) (second_of_day / 60 % 60);
    result.hour = cast(s32) (second_of_day / 3600);
    result.day_of_month = cast(s32) day;
    result.month = cast(s32) (month - 1);
    result.years_since_1900 = cast,no_check(s32) (year - 1900);
    result.day_of_week = cast(s32) weekday;
    result.day_of_year = cast(s32) day_in_year;
    return result;
}

#program_export "gettimeofday"
wasi_gettimeofday :: (result: *Seconds_And_Microseconds, zone: *void) -> s32 #c_call {
    now: Seconds_And_Nanoseconds;
    if wasi_clock_gettime(0, *now) != 0 return -1;
    if result {
        result.seconds = now.seconds;
        result.microseconds = now.nanoseconds / 1000;
    }
    return 0;
}

Seconds_And_Microseconds :: struct {
    seconds: s64;
    microseconds: s64;
}

// ---------------------------------------------------------------------------------------------
// Threads: WASI preview 1 has none. The program is the only thread, so locks always succeed and
// nothing can ever signal a condition variable: a timed wait sleeps out its deadline, an untimed
// one reports the deadlock. Creating a thread fails, so `thread_start` returns false.

#program_export "pthread_create"
wasi_pthread_create :: (thread: *u64, attributes: *void, start: *void, argument: *void) -> s32 #c_call {
    return EAGAIN;
}

#program_export "pthread_join"
wasi_pthread_join :: (thread: u64, result: **void) -> s32 #c_call {
    return ESRCH;
}

#program_export "pthread_tryjoin_np"
wasi_pthread_tryjoin_np :: (thread: u64, result: **void) -> s32 #c_call {
    return ESRCH;
}

#program_export "pthread_mutexattr_init"
wasi_pthread_mutexattr_init :: (attributes: *void) -> s32 #c_call {
    return 0;
}

#program_export "pthread_mutexattr_settype"
wasi_pthread_mutexattr_settype :: (attributes: *void, kind: s32) -> s32 #c_call {
    return 0;
}

#program_export "pthread_mutexattr_destroy"
wasi_pthread_mutexattr_destroy :: (attributes: *void) -> s32 #c_call {
    return 0;
}

#program_export "pthread_mutex_init"
wasi_pthread_mutex_init :: (mutex: *void, attributes: *void) -> s32 #c_call {
    return 0;
}

#program_export "pthread_mutex_destroy"
wasi_pthread_mutex_destroy :: (mutex: *void) -> s32 #c_call {
    return 0;
}

#program_export "pthread_mutex_lock"
wasi_pthread_mutex_lock :: (mutex: *void) -> s32 #c_call {
    return 0;
}

#program_export "pthread_mutex_trylock"
wasi_pthread_mutex_trylock :: (mutex: *void) -> s32 #c_call {
    return 0;
}

#program_export "pthread_mutex_unlock"
wasi_pthread_mutex_unlock :: (mutex: *void) -> s32 #c_call {
    return 0;
}

#program_export "pthread_cond_init"
wasi_pthread_cond_init :: (condition: *void, attributes: *void) -> s32 #c_call {
    return 0;
}

#program_export "pthread_cond_destroy"
wasi_pthread_cond_destroy :: (condition: *void) -> s32 #c_call {
    return 0;
}

#program_export "pthread_cond_signal"
wasi_pthread_cond_signal :: (condition: *void) -> s32 #c_call {
    return 0;
}

#program_export "pthread_cond_broadcast"
wasi_pthread_cond_broadcast :: (condition: *void) -> s32 #c_call {
    return 0;
}

#program_export "pthread_cond_wait"
wasi_pthread_cond_wait :: (condition: *void, mutex: *void) -> s32 #c_call {
    return EDEADLK;
}

// `deadline` is CLOCK_REALTIME, as pthread_cond_timedwait takes it by default.
#program_export "pthread_cond_timedwait"
wasi_pthread_cond_timedwait :: (condition: *void, mutex: *void, deadline: *Seconds_And_Nanoseconds) -> s32 #c_call {
    if !deadline return EINVAL;
    now: Seconds_And_Nanoseconds;
    if wasi_clock_gettime(0, *now) != 0 return EINVAL;
    remaining: Seconds_And_Nanoseconds;
    remaining.seconds = deadline.seconds - now.seconds;
    remaining.nanoseconds = deadline.nanoseconds - now.nanoseconds;
    if remaining.nanoseconds < 0 {
        remaining.nanoseconds += 1_000_000_000;
        remaining.seconds -= 1;
    }
    if remaining.seconds >= 0 wasi_nanosleep(*remaining, null);
    return ETIMEDOUT;
}

// ---------------------------------------------------------------------------------------------
// Processes: WASI preview 1 cannot start, signal or wait for one and has no pipes, so these
// fail (ENOSYS; no children to wait for or signal). A program that links Process still runs
// and sees the failure where it starts one.

#program_export "fork"
wasi_fork :: () -> s32 #c_call {
    error_number = ENOSYS;
    return -1;
}

#program_export "execvp"
wasi_execvp :: (file: *u8, arguments: **u8) -> s32 #c_call {
    error_number = ENOSYS;
    return -1;
}

#program_export "waitpid"
wasi_waitpid :: (process: s32, status: *s32, options: s32) -> s32 #c_call {
    error_number = ECHILD;
    return -1;
}

#program_export "kill"
wasi_kill :: (process: s32, signal: s32) -> s32 #c_call {
    error_number = ESRCH;
    return -1;
}

#program_export "pipe"
wasi_pipe :: (ends: *[2] s32) -> s32 #c_call {
    error_number = ENOSYS;
    return -1;
}

#program_export "socketpair"
wasi_socketpair :: (domain: s32, kind: s32, protocol: s32, ends: *[2] s32) -> s32 #c_call {
    error_number = ENOSYS;
    return -1;
}

#program_export "dup2"
wasi_dup2 :: (from: s32, to: s32) -> s32 #c_call {
    error_number = ENOSYS;
    return -1;
}

// Waiting on no descriptors is a sleep; WASI's poll_oneoff could watch descriptors, but nothing
// here produces one worth polling (no pipes or sockets).
#program_export "poll"
wasi_poll :: (descriptors: *void, count: u64, timeout_milliseconds: s32) -> s32 #c_call {
    if count != 0 {
        error_number = ENOSYS;
        return -1;
    }
    if timeout_milliseconds > 0 {
        interval := Seconds_And_Nanoseconds.{ timeout_milliseconds / 1000, (timeout_milliseconds % 1000) * 1_000_000 };
        wasi_nanosleep(*interval, null);
    }
    return 0;
}

#program_export "usleep"
wasi_usleep :: (microseconds: u32) -> s32 #c_call {
    interval := Seconds_And_Nanoseconds.{ microseconds / 1_000_000, (microseconds % 1_000_000) * 1000 };
    return wasi_nanosleep(*interval, null);
}

// Close-on-exec is moot without exec: F_GETFD reads 0 and F_SETFD succeeds. The rest are not
// supported. `fcntl` is variadic in C, so its third argument arrives as a pointer.
#program_export "fcntl"
wasi_fcntl :: (fd: s32, command: s32, extra: *void) -> s32 #c_call {
    if command == 1 || command == 2 return 0;
    error_number = ENOSYS;
    return -1;
}

#program_export "__errno_location"
wasi_errno_location :: () -> *s32 #c_call {
    return *error_number;
}

error_number: s32;
ENOENT :: 2;
ESRCH :: 3;
ECHILD :: 10;
EAGAIN :: 11;
EINVAL :: 22;
ERANGE :: 34;
EDEADLK :: 35;
ETIMEDOUT :: 110;

wasi_poll_oneoff :: (subscriptions: u32, events: u32, count: u32, happened: u32) -> s32 #foreign wasi "poll_oneoff";

// WASI's `subscription` with its `subscription_clock` variant (48 bytes) and its `event` (32 bytes).
Clock_Subscription :: struct {
    user_data: u64;
    kind: u8;
    kind_padding: [7] u8;
    clock_id: u32;
    clock_padding: u32;
    timeout: u64;
    precision: u64;
    flags: u16;
    flags_padding: [6] u8;
}

Poll_Event :: struct {
    user_data: u64;
    error: u16;
    kind: u8;
    kind_padding: [5] u8;
    bytes: u64;
    flags: u16;
    flags_padding: [6] u8;
}

#assert size_of(Clock_Subscription) == 48 && size_of(Poll_Event) == 32;

// Linux's `struct tm`, which the stdlib's `OS == .WASM` code reads.
Broken_Down_Time :: struct {
    second: s32;
    minute: s32;
    hour: s32;
    day_of_month: s32;
    month: s32;
    years_since_1900: s32;
    day_of_week: s32;
    day_of_year: s32;
    daylight_saving: s32;
    gmt_offset: s64;
    zone: *u8;
}

#assert size_of(Broken_Down_Time) == 56;

environment: **u8;
environment_ready: bool;

load_environment :: () #no_context {
    count, text_bytes: u32;
    if wasi_environ_sizes_get(offset32(*count), offset32(*text_bytes)) != 0 return;
    table := cast(*u64) heap_take(cast(u64) (count + 1) * 8);
    text := heap_take(cast(u64) text_bytes + 1);
    if !table || !text return;
    if wasi_environ_get(offset32(table), offset32(text)) != 0 return;
    offsets := cast(*u32) table;
    for < i: 0..cast(s64) count - 1 table[i] = cast(u64) offsets[i];
    table[count] = 0;
    environment = cast(**u8) table;
}

write_all :: (fd: s32, data: *void, count: s64) #no_context {
    done: s64;
    while done < count {
        step := wasi_write(fd, cast(*u8) data + done, cast(u64) (count - done));
        if step <= 0 return;
        done += step;
    }
}

c_length :: (text: *u8) -> s64 #no_context {
    n: s64;
    while text[n] n += 1;
    return n;
}

bytes_equal :: (a: *u8, b: *u8, count: s64) -> bool #no_context {
    for 0..count - 1 if a[it] != b[it] return false;
    return true;
}

// ---------------------------------------------------------------------------------------------
// Memory

#program_export "memmove"
wasi_memmove :: (destination: *void, source: *void, count: u64) -> *void #c_call {
    // Becomes `memory.copy` (jaic always builds wasm with bulk memory), never a call back here.
    memcpy(destination, source, cast(s64) count);
    return destination;
}

#program_export "memcpy"
wasi_memcpy :: (destination: *void, source: *void, count: u64) -> *void #c_call {
    memcpy(destination, source, cast(s64) count);
    return destination;
}

#program_export "memset"
wasi_memset :: (destination: *void, value: s32, count: u64) -> *void #c_call {
    memset(destination, cast,no_check(u8) value, cast(s64) count);
    return destination;
}

// The signature the backend's `memcmp` intrinsic calls with.
#program_export "memcmp"
wasi_memcmp :: (a: *void, b: *void, count: s64) -> s32 #c_call {
    left := cast(*u8) a;
    right := cast(*u8) b;
    for 0..count - 1 {
        if left[it] != right[it] return cast(s32) left[it] - cast(s32) right[it];
    }
    return 0;
}

#program_export "strlen"
wasi_strlen :: (text: *u8) -> u64 #c_call {
    return cast(u64) c_length(text);
}

// The heap: blocks are powers of two from 32 bytes up, each starting with a 16-byte tag that
// records its size class, which keeps the returned addresses 16-aligned. A freed block goes
// on the list of its class and is handed out again as is; fresh blocks are cut from the top
// of the heap, which grows memory in steps of at least 1 MiB. Nothing is ever returned to the
// host. The program is single threaded, so there is no locking.

TAG_BYTES :: 16;
SMALLEST_CLASS :: 5;
CLASS_COUNT :: 48;
PAGE_BYTES :: 65536;
GROWTH_PAGES :: 16;

Free_Block :: struct {
    next: *Free_Block;
}

free_by_class: [CLASS_COUNT] *Free_Block;
heap_cursor: u64;
heap_limit: u64;

class_for :: (bytes: u64) -> s64 #no_context {
    class := SMALLEST_CLASS;
    while (cast(u64) 1 << cast(u64) class) < bytes class += 1;
    return class;
}

// Raw memory from the top of the heap, 16-aligned; null when memory cannot grow.
heap_take :: (bytes: u64) -> *u8 #no_context {
    if heap_cursor == 0 {
        heap_cursor = (cast(u64) *__heap_base + 15) & ~cast(u64) 15;
        heap_limit = cast(u64) memory_page_count(0) * PAGE_BYTES;
    }
    size := (bytes + 15) & ~cast(u64) 15;
    if size > heap_limit || heap_cursor > heap_limit - size {
        missing := heap_cursor + size - heap_limit;
        pages := cast(s64) ((missing + PAGE_BYTES - 1) / PAGE_BYTES);
        if pages < GROWTH_PAGES pages = GROWTH_PAGES;
        if memory_add_pages(0, pages) < 0 return null;
        heap_limit += cast(u64) pages * PAGE_BYTES;
    }
    result := cast(*u8) heap_cursor;
    heap_cursor += size;
    return result;
}

#program_export "malloc"
wasi_malloc :: (size: u64) -> *void #c_call {
    if size > (cast(u64) 1 << 46) return null;
    class := class_for(size + TAG_BYTES);
    block := cast(*u8) free_by_class[class];
    if block {
        free_by_class[class] = (cast(*Free_Block) block).next;
    } else {
        block = heap_take(cast(u64) 1 << cast(u64) class);
        if !block return null;
    }
    (cast(*s64) block).* = class;
    return block + TAG_BYTES;
}

#program_export "free"
wasi_free :: (memory: *void) #c_call {
    if !memory return;
    block := cast(*u8) memory - TAG_BYTES;
    class := (cast(*s64) block).*;
    entry := cast(*Free_Block) block;
    entry.next = free_by_class[class];
    free_by_class[class] = entry;
}

#program_export "realloc"
wasi_realloc :: (memory: *void, size: u64) -> *void #c_call {
    if !memory return wasi_malloc(size);
    if size == 0 {
        wasi_free(memory);
        return null;
    }
    class := (cast(*s64) (cast(*u8) memory - TAG_BYTES)).*;
    room := (cast(u64) 1 << cast(u64) class) - TAG_BYTES;
    if size <= room return memory;
    moved := wasi_malloc(size);
    if !moved return null;
    memcpy(moved, memory, cast(s64) room);
    wasi_free(memory);
    return moved;
}

#program_export "calloc"
wasi_calloc :: (count: u64, size: u64) -> *void #c_call {
    if size != 0 && count > 0xffff_ffff_ffff_ffff / size return null;
    total := count * size;
    memory := wasi_malloc(total);
    if memory memset(memory, 0, cast(s64) total);
    return memory;
}
