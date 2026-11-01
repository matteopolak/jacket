// Nonblocking pseudoterminal sessions for macOS and Linux.
// @experimental

#import "Basic";

// A child process and its PTY. Close with `close_session`; do not copy live sessions.
Session :: struct {
    // Nonblocking master descriptor, or -1 when closed.
    fd: s32 = -1;
    // Child session leader, or -1 when no child was started.
    pid: s32 = -1;
    // True once poll_exit has reaped the child.
    exited: bool;
    // Native waitpid status, valid after exited becomes true.
    status: s32;
}

// Starts `shell` as an interactive shell, or with `-c command` when command is nonempty.
// The child inherits the environment with TERM=xterm-256color. Call before starting other threads.
// Returns false on setup failure. Windows returns unsupported without importing POSIX.
// ```jai,no_run
// session, ok := open_session("/bin/sh", "printf hello", 80, 24);
// if ok close_session(*session);
// ```
open_session :: (shell: string, command: string = "", cols: int = 80, rows: int = 24) -> Session, bool {
    result: Session;

    #if OS == .MACOS || OS == .LINUX {
        if cols < 1 || rows < 1 || cols > 65535 || rows > 65535 return result, false;

        fd := posix_openpt(P.O_RDWR | P.O_NOCTTY);
        if fd < 0 return result, false;

        if grantpt(fd) != 0 || unlockpt(fd) != 0 {
            P.close(fd);
            return result, false;
        }

        name := ptsname(fd);
        if !name {
            P.close(fd);
            return result, false;
        }

        // Prepare all storage before fork: the child only calls async-signal-safe libc APIs.
        shell_z := to_c_string(shell);
        command_z := to_c_string(command);

        defer free(shell_z);
        defer free(command_z);
        args: [4] *u8;
        args[0] = shell_z;
        args[1] = ifx command.count then cast(*u8) "-c\0".data else cast(*u8) "-i\0".data;
        if command.count args[2] = command_z;

        // Build envp before fork; keep inherited strings borrowed until exec.
        env: [..] *u8;

        defer array_free(env);
        #if OS == .MACOS inherited := native_environment().*;
        else inherited := P.environ;
        index := 0;
        while inherited[index] {
            entry := inherited[index];
            if !begins_with(to_string(entry), "TERM=") array_add(*env, entry);
            index += 1;
        }

        array_add(*env, cast(*u8) "TERM=xterm-256color\0".data);
        array_add(*env, cast(*u8) null);

        slave := P.open(name, P.O_RDWR | P.O_NOCTTY);
        if slave < 0 {
            P.close(fd);
            return result, false;
        }

        size := Window_Size.{ cast(u16) rows, cast(u16) cols, 0, 0 };
        if native_ioctl(fd, SET_SIZE, *size) != 0 {
            P.close(slave);
            P.close(fd);
            return result, false;
        }

        pid := P.fork();
        if pid == 0 {
            P.close(fd);
            if P.setsid() < 0 P._exit(126);
            if native_ioctl(slave, SET_CTTY, cast(*void) null) < 0 P._exit(126);
            for i: 0..2 if P.dup2(slave, cast(s32) i) < 0 P._exit(126);
            if slave > 2 P.close(slave);

            P.execve(shell_z, args.data, env.data);
            P._exit(127);
        }

        P.close(slave);

        if pid < 0 {
            P.close(fd);
            return result, false;
        }

        if P.fcntl(fd, P.F_SETFL, P.O_NONBLOCK) < 0 || P.fcntl(fd, P.F_SETFD, P.FD_CLOEXEC) < 0 {
            P.close(fd);
            P.kill(pid, P.SIGKILL);
            status: s32;

            P.waitpid(pid, *status, 0);

            return result, false;
        }

        result.fd = fd;
        result.pid = pid;

        return result, true;
    } else return result, false;
}

// Reads available output: positive byte count, zero for would-block, -1 for EOF/error.
// The caller owns `buffer`. `poll_exit` separately reports the child's exit status.
read_session :: (session: *Session, buffer: [] u8) -> int {
    #if OS == .MACOS || OS == .LINUX {
        n := P.read(session.fd, buffer.data, cast(u64) buffer.count);
        if n < 0 && (errno_address().* == P.EAGAIN || errno_address().* == P.EINTR) return 0;
        if n <= 0 return -1;
        return n;
    } else return -1;
}

// Writes as much input as fits without blocking. Returns bytes written (zero on would-block).
// Keep the unwritten suffix and retry next frame.
write_session :: (session: *Session, text: string) -> int {
    #if OS == .MACOS || OS == .LINUX {
        n := P.write(session.fd, text.data, cast(u64) text.count);
        return max(0, n);
    } else return 0;
}

// Changes the PTY size. The kernel sends SIGWINCH to its foreground process group.
resize_session :: (session: *Session, cols: int, rows: int) -> bool {
    #if OS == .MACOS || OS == .LINUX {
        if cols < 1 || rows < 1 || cols > 65535 || rows > 65535 return false;
        size := Window_Size.{ cast(u16) rows, cast(u16) cols, 0, 0 };
        return native_ioctl(session.fd, SET_SIZE, *size) == 0;
    } else return false;
}

// Reaps the child without blocking. `status` is the native wait status, not an exit code.
poll_exit :: (session: *Session) -> bool {
    #if OS == .MACOS || OS == .LINUX {
        if !session.exited && session.pid > 0 {
            if P.waitpid(session.pid, *session.status, P.WNOHANG) == session.pid session.exited = true;
        }
    }

    return session.exited;
}

// Closes the master, terminates a remaining child and reaps it. Safe on an empty session.
close_session :: (session: *Session) {
    #if OS == .MACOS || OS == .LINUX {
        if session.fd >= 0 P.close(session.fd);

        if session.pid > 0 && !session.exited {
            P.kill(-session.pid, P.SIGHUP);
            P.kill(session.pid, P.SIGKILL);
            P.waitpid(session.pid, *session.status, 0);
        }
    }

    session.* = .{};
}

#scope_file
#if OS == .MACOS || OS == .LINUX {
    #import "String";
    P :: #import "POSIX";

    libc :: #library,system "c";
    posix_openpt :: (flags: s32) -> s32 #foreign libc;
    grantpt :: (fd: s32) -> s32 #foreign libc;
    unlockpt :: (fd: s32) -> s32 #foreign libc;
    ptsname :: (fd: s32) -> *u8 #foreign libc;
    native_ioctl :: (fd: s32, request: u64, args: ..Any) -> s32 #foreign libc "ioctl";

    Window_Size :: struct {
        rows: u16;
        cols: u16;
        x: u16;
        y: u16;
    }

    #if OS == .MACOS {
        native_environment :: () -> ***u8 #foreign libc "_NSGetEnviron";
        SET_SIZE :: 0x80087467;
        SET_CTTY :: 0x20007461;
        errno_address :: () -> *s32 #foreign libc "__error";
    } else {
        SET_SIZE :: 0x5414;
        SET_CTTY :: 0x540e;
        errno_address :: () -> *s32 #foreign libc "__errno_location";
    }
}
