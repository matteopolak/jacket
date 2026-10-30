// Service: compile-time services. A jaic extension, not official Jai: other Jai compilers have no
// such module, and it may change between jaic releases. See docs/extensions/service.md.
//
// A service is a long-lived program that compile-time code asks questions: one that keeps a
// database connection open, say, so `#run` code can learn a query's types without connecting
// on every compile. The compiler starts it the first time it is called (`jaic <file> -interp`),
// keeps it running between calls, and restarts it when its sources change.
//
// The service side (service.jai):
//
//     Service :: #import "Extensions/Service";
//     main :: () {
//         Service.serve((request: string) -> Service.Status, string {
//             return .OK, to_upper_copy(request);
//         });
//     }
//
// The compile-time side:
//
//     SERVICE :: #run tprint("%service.jai", #filepath);
//     #run {
//         status, answer := Service.call(SERVICE, "hello");
//         if status == {
//             case .OK;          print("%\n", answer);
//             case .PENDING;     Service.unknown("asking the service");   // editor only
//             case .UNAVAILABLE; Service.unknown(answer);
//             case .FAILED;      compiler_report(answer);
//         }
//     }
//
// In a build, `call` waits for the answer. In the editor (a trusted workspace) it answers
// `.PENDING` while the service works, and the editor checks the file again once the answer is
// in. Untrusted workspaces run no compile-time code at all.

#import "Basic";

Status :: enum s64 {
    OK :: 0; // the answer is the service's reply
    PENDING :: 1; // editor only: the reply is not in yet; the file is checked again later
    UNAVAILABLE :: 2; // no service here (no jaic to run it, a sandbox, or the service gave up)
    FAILED :: 3; // the service crashed, timed out, ran out of memory or reported an error
}

// Asks the service in the file `service` (absolute, or relative to the compiler's working
// directory). The reply lives until the end of the compile.
call :: (service: string, request: string) -> Status, string {
    reply: string;
    status := compile_time_service(service, request, *reply);
    return cast(Status) status, reply;
}

// Stops the compile-time code that called it: its result is not known here (a reply was
// pending or unavailable). A build reports `message` as an error; the editor reports nothing
// and keeps checking the rest of the file.
unknown :: (message: string) {
    compile_time_unknown(message);
}

// The service side: answers each request with `handler` until the compiler closes the input.
// Return `.OK` with the reply, `.FAILED` with an error message, or `.UNAVAILABLE` with the
// reason the service cannot answer here (no database configured, say). Whatever the handler
// prints or logs goes to stderr: stdout carries the answers.
serve :: (handler: (request: string) -> (Status, string)) {
    if !take_stdout() return;

    while true {
        length_bytes: [8] u8;
        if !read_exactly(length_bytes.data, 8) return;

        length := (cast(*s64) length_bytes.data).*;
        if length < 0 return;

        request := alloc_string(length);

        defer free(request);
        if !read_exactly(request.data, length) return;

        status, reply := handler(request);
        if status == .PENDING status = .FAILED; // only the compiler says pending

        header: [9] u8;
        (cast(*s64) header.data).* = reply.count + 1;
        header[8] = cast(u8) status;
        if !write_all(header.data, 9) return;
        if !write_all(reply.data, reply.count) return;

        reset_temporary_storage();
    }
}

#scope_module

compile_time_service :: (service: string, request: string, reply: *string) -> s64 #compiler #no_context {
    return xx Status.UNAVAILABLE;
}

compile_time_unknown :: (message: string) #compiler #no_context {}

#if OS == .WINDOWS {
    Windows :: #import "Windows";

    read_exactly :: (data: *u8, count: s64) -> bool {
        input := Windows.GetStdHandle(Windows.STD_INPUT_HANDLE);
        done := 0;

        while done < count {
            got: u32;
            want := cast(u32) min(count - done, 1 << 30);
            if !Windows.ReadFile(input, data + done, want, *got) || got == 0 return false;
            done += got;
        }

        return true;
    }

    kernel32 :: #library,system "kernel32"; @native(external)
    SetStdHandle :: (handle: s32, value: Windows.HANDLE) -> s32 #foreign kernel32;

    answers: Windows.HANDLE;

    // Keeps stdout for the answers and points the process's stdout at stderr.
    take_stdout :: () -> bool {
        answers = Windows.GetStdHandle(Windows.STD_OUTPUT_HANDLE);
        return SetStdHandle(Windows.STD_OUTPUT_HANDLE, Windows.GetStdHandle(Windows.STD_ERROR_HANDLE)) != 0;
    }

    write_all :: (data: *u8, count: s64) -> bool {
        output := answers;
        done := 0;

        while done < count {
            put: u32;
            want := cast(u32) min(count - done, 1 << 30);
            if !Windows.WriteFile(output, data + done, want, *put, null) || put == 0 return false;
            done += put;
        }

        return true;
    }
} else {
    libc :: #system_library "libc"; @native(external)
    c_read :: (fd: s32, buffer: *void, count: u64) -> s64 #foreign libc "read";
    c_write :: (fd: s32, buffer: *void, count: u64) -> s64 #foreign libc "write";
    c_dup :: (fd: s32) -> s32 #foreign libc "dup";
    c_dup2 :: (fd: s32, to: s32) -> s32 #foreign libc "dup2";

    answers: s32 = 1;

    // Keeps stdout for the answers and points file descriptor 1 at stderr.
    take_stdout :: () -> bool {
        answers = c_dup(1);
        return answers >= 0 && c_dup2(2, 1) == 1;
    }

    read_exactly :: (data: *u8, count: s64) -> bool {
        done := 0;
        while done < count {
            got := c_read(0, data + done, cast(u64) (count - done));
            if got <= 0 return false;
            done += got;
        }

        return true;
    }

    write_all :: (data: *u8, count: s64) -> bool {
        done := 0;
        while done < count {
            put := c_write(answers, data + done, cast(u64) (count - done));
            if put <= 0 return false;
            done += put;
        }

        return true;
    }
}
