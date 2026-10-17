// Win32 engine uses the canonical Process records as authoritative handle state.
Stream :: Types.Process.Pipe;
Child :: Types.Process;

launch :: (child: *Child, arguments: [] string, working_directory := "", capture := false, kill_on_parent_exit := true, quote_arguments := true) -> bool {
    if child.info.hProcess || valid_handle(child.job_handle) || valid_handle(child.input.handle) || valid_handle(child.output.handle) || valid_handle(child.error.handle) return false;
    if arguments.count == 0 || arguments[0].count == 0 return false;

    command_builder: String_Builder;

    defer free_buffers(*command_builder);

    for argument, i: arguments {
        if i > 0 append(*command_builder, cast(u8) 32);
        if quote_arguments append_quoted(*command_builder, argument);
        else append(*command_builder, argument);
    }

    if command_builder.failed return false;

    command_text := builder_to_string(*command_builder);

    defer free(command_text);
    command := Text.utf16_path(command_text);

    defer free(command);
    if !command return false;

    command_length: s64;
    while command[command_length] != 0 command_length += 1;
    if command_length >= 32767 return false;

    directory: *u16;

    if working_directory.count > 0 {
        directory = Text.utf16_path(working_directory);
        if !directory return false;
    }

    defer free(directory);

    result: Child;
    committed := false;

    defer if !committed destroy(*result);
    clients: [3] W.HANDLE;

    defer for handle: clients if valid_handle(handle) checked_close(handle);
    startup: Startup_Extended;
    startup.info.cb = cast(s32) size_of(W.STARTUPINFOW);
    attribute_memory: *void;
    attributes_initialized := false;

    defer {
        if attributes_initialized DeleteProcThreadAttributeList(attribute_memory);
        free(attribute_memory);
    }

    flags: u32 = 4; // CREATE_SUSPENDED, until job ownership has been established.
    inherit: W.BOOL;

    if capture {
        ok: bool;
        ok, clients[0] = pipe_pair(*result.input, true);
        if !ok return false;

        ok, clients[1] = pipe_pair(*result.output, false);
        if !ok return false;

        ok, clients[2] = pipe_pair(*result.error, false);
        if !ok return false;

        startup.info.hStdInput = clients[0];
        startup.info.hStdOutput = clients[1];
        startup.info.hStdError = clients[2];
        startup.info.dwFlags = 0x100;
        startup.info.cb = cast(s32) size_of(Startup_Extended);
        bytes: u64;
        sized := InitializeProcThreadAttributeList(null, 1, 0, *bytes);

        if sized == 0 && W.GetLastError() != 122 return false;
        if bytes == 0 || bytes > 0x7fffffffffffffff return false;

        attribute_memory = alloc(cast(s64) bytes);
        if !attribute_memory return false;
        if InitializeProcThreadAttributeList(attribute_memory, 1, 0, *bytes) == 0 return false;

        attributes_initialized = true;
        if UpdateProcThreadAttribute(attribute_memory, 0, 0x20002, clients.data, 3 * size_of(W.HANDLE), null, null) == 0 return false;

        startup.attributes = attribute_memory;
        flags |= 0x80000; // EXTENDED_STARTUPINFO_PRESENT: inherit exactly these three handles.
        inherit = 1;
    } else {
        result.input.eof = true;
        result.output.eof = true;
        result.error.eof = true;
    }

    if kill_on_parent_exit {
        result.job_handle = W.CreateJobObjectA(null, null);
        if !result.job_handle return false;

        limits: W.JOBOBJECT_EXTENDED_LIMIT_INFORMATION;
        limits.BasicLimitInformation.LimitFlags = cast(W.Job_Object_Limit) 0x2000;
        if W.SetInformationJobObject(result.job_handle, .ExtendedLimitInformation, *limits, cast(u32) size_of(type_of(limits))) == 0 return false;
    }

    process: W.PROCESS_INFORMATION;
    if W.CreateProcessW(null, command, null, null, inherit, cast(W.Creation_Flags) flags, null, directory, *startup.info, *process) == 0 return false;

    result.info = process;

    if valid_handle(result.job_handle) && W.AssignProcessToJobObject(result.job_handle, result.info.hProcess) == 0 {
        terminate_and_wait(result.info.hProcess);
        return false;
    }

    if W.ResumeThread(process.hThread) == -1 {
        terminate_and_wait(result.info.hProcess);
        return false;
    }

    child.* = result;
    committed = true;

    return true;
}

wait :: (child: *Child, timeout_ms: s64 = -1) -> success: bool, done: bool, exit_code: u32 {
    if !child.info.hProcess return false, false, 0;

    handles := W.HANDLE.[child.info.hProcess];
    status := wait_handles(1, handles.data, timeout_ms);

    if status == 258 return true, false, 0;
    if status != 0 return false, false, 0;

    code: u32;
    if W.GetExitCodeProcess(child.info.hProcess, *code) == 0 return false, false, 0;

    return true, true, code;
}

terminate :: (child: *Child) -> bool {
    if !child.info.hProcess return false;

    ok, done := wait(child, 0);
    if !ok return false;
    if done return true;

    return W.TerminateProcess(child.info.hProcess, 1) != 0;
}

close_input :: (child: *Child) {
    close_stream(*child.input);
} @Releases

initialize_pipe :: (pipe: *Stream, is_input: bool) -> success: bool, client_handle: W.HANDLE = W.INVALID_HANDLE_VALUE {
    return pipe_pair(pipe, is_input);
}

destroy_pipe :: (pipe: *Stream) {
    close_stream(pipe);
} @Releases

read_pipe :: (pipe: *Stream, buffer: [] u8) -> success: bool, bytes_read: int {
    ok, count := begin_read(pipe, buffer);
    finished, extra := finish_read(pipe);
    return ok && finished, count + extra;
}

destroy :: (child: *Child) {
    close_stream(*child.input);
    close_stream(*child.output);
    close_stream(*child.error);

    if valid_handle(child.job_handle) {
        checked_close(child.job_handle);
        child.job_handle = W.INVALID_HANDLE_VALUE;
    }

    if child.info.hThread checked_close(child.info.hThread);
    if child.info.hProcess checked_close(child.info.hProcess);

    child.info = .{};
} @Releases

write_input :: (child: *Child, data: [] u8) -> bool, s64 {
    stream := *child.input;
    if !valid_handle(stream.handle) || stream.eof || operation_state(stream).pending return false, 0;

    total: s64;
    while total < data.count {
        reset_operation(stream);
        chunk := cast(u32) min(data.count - total, 0x7fffffff);
        written: u32;
        ok := W.WriteFile(stream.handle, data.data + total, chunk, null, stream.overlapped);

        if ok == 0 {
            if W.GetLastError() != 997 return false, total;

            operation_state(stream).pending = true;

            if GetOverlappedResult(stream.handle, stream.overlapped, *written, 1) == 0 {
                operation_state(stream).pending = false;
                return false, total;
            }

            operation_state(stream).pending = false;
        } else if GetOverlappedResult(stream.handle, stream.overlapped, *written, 0) == 0 return false, total;

        total += written;
        if written == 0 return false, total;
    }

    return true, total;
}

read_pair :: (child: *Child, output: [] u8, error: [] u8, timeout_ms: s64 = -1) -> bool, s64, s64 {
    out_ok, out_count := begin_read(*child.output, output);
    err_ok, err_count := begin_read(*child.error, error);
    handles: [2] W.HANDLE;
    count: u32;

    if has_pending(*child.output) {
        handles[count] = child.output.overlapped.hEvent;
        count += 1;
    }

    if has_pending(*child.error) {
        handles[count] = child.error.overlapped.hEvent;
        count += 1;
    }

    waited := true;

    if count > 0 && out_count == 0 && err_count == 0 && out_ok && err_ok {
        status := wait_handles(count, handles.data, timeout_ms);
        waited = status < count || status == 258;
    }

    out_finished, out_extra := finish_read(*child.output);
    err_finished, err_extra := finish_read(*child.error);

    return out_ok && err_ok && waited && out_finished && err_finished, out_count + out_extra, err_count + err_extra;
}

#scope_module
#import "Basic";
#load "../process/arguments.jai";
W :: #import "Windows";
Types :: #import "internal/process/windows-types";
Text :: #import "internal/files/windows-text";

Startup_Extended :: struct {
    info: W.STARTUPINFOW;
    attributes: *void;
}

#assert size_of(W.OVERLAPPED) == 32;
#assert size_of(W.STARTUPINFOW) == 104;
#assert size_of(Startup_Extended) == 112;
#assert size_of(W.PROCESS_INFORMATION) == 24;
#assert size_of(W.SECURITY_ATTRIBUTES) == 24;
#assert size_of(W.JOBOBJECT_EXTENDED_LIMIT_INFORMATION) == 144;

kernel32 :: #system_library "kernel32";
InitializeProcThreadAttributeList :: (list: *void, count: u32, flags: u32, size: *u64) -> s32 #foreign kernel32;
UpdateProcThreadAttribute :: (list: *void, flags: u32, attribute: u64, value: *void, size: u64, previous: *void, returned_size: *u64) -> s32 #foreign kernel32;
DeleteProcThreadAttributeList :: (list: *void) #foreign kernel32;
ResetEvent :: (event: W.HANDLE) -> s32 #foreign kernel32;
GetOverlappedResult :: (file: W.HANDLE, operation: *W.OVERLAPPED, count: *u32, wait: s32) -> W.BOOL #foreign kernel32;
CancelIoEx :: (file: W.HANDLE, operation: *W.OVERLAPPED) -> s32 #foreign kernel32;

pipe_counter: s64;

Operation_State :: struct {
    native: W.OVERLAPPED;
    allocator: Allocator;
    pending: bool;
}

operation_state :: (stream: *Stream) -> *Operation_State {
    assert(stream.overlapped != null, "pipe has no overlapped state");
    // Native state is the first member, so the public pointer has a stable address.
    return cast(*Operation_State) stream.overlapped;
}

has_pending :: (stream: *Stream) -> bool {
    return stream.overlapped != null && operation_state(stream).pending;
}

valid_handle :: (handle: W.HANDLE) -> bool {
    return handle != null && handle != W.INVALID_HANDLE_VALUE;
}

checked_close :: (handle: W.HANDLE) {
    assert(W.CloseHandle(handle) != 0, "Windows process handle cleanup failed");
} @Releases

wait_handles :: (count: u32, handles: *W.HANDLE, milliseconds: s64) -> u32 {
    if milliseconds == 0 return W.WaitForMultipleObjects(count, handles, 0, 0);
    if milliseconds < 0 return W.WaitForMultipleObjects(count, handles, 0, 0xffffffff);

    remaining := milliseconds;

    while remaining > 0 {
        // INFINITE reserves one DWORD value. Repeat finite chunks rather than
        // truncating a public 64-bit timeout to the native wait's range.
        chunk := cast(u32) min(remaining, cast(s64) 0xfffffffe);
        status := W.WaitForMultipleObjects(count, handles, 0, chunk);

        if status != 258 return status;

        remaining -= chunk;
    }

    // Every finite chunk returned WAIT_TIMEOUT; the requested budget is spent.
    return 258;
}

terminate_and_wait :: (process: W.HANDLE) {
    if W.TerminateProcess(process, 1) == 0
        assert(W.WaitForSingleObject(process, 0) == 0, "failed to terminate partially initialized process");
    assert(W.WaitForSingleObject(process, 0xffffffff) == 0, "failed to collect partially initialized process");
}

pipe_pair :: (stream: *Stream, is_input: bool) -> bool, W.HANDLE {
    if valid_handle(stream.handle) || stream.overlapped return false, W.INVALID_HANDLE_VALUE;

    owner := context.allocator;
    state := cast(*Operation_State) owner.proc(.ALLOCATE, size_of(Operation_State), 0, null, owner.data);

    if !state return false, W.INVALID_HANDLE_VALUE;

    state.* = .{};
    state.allocator = owner;
    stream.overlapped = *state.native;
    stream.eof = false;
    committed := false;

    defer if !committed close_stream(stream);
    _, serial := compare_and_swap(*pipe_counter, cast(s64) 0, cast(s64) 0);

    while true {
        assert(serial < 0x7fffffffffffffff, "process pipe serial exhausted");
        advanced, old := compare_and_swap(*pipe_counter, serial, serial + 1);
        if advanced break;
        serial = old;
    }

    name := Text.utf16_path(tprint("\\\\.\\pipe\\cleanroom-io-%-%", W.GetCurrentProcessId(), serial));

    defer free(name);
    access: u32 = 1; // PIPE_ACCESS_INBOUND: parent reads child's output.
    client_access: u32 = W.GENERIC_WRITE;

    if is_input {
        access = 2;
        client_access = W.GENERIC_READ;
    }

    stream.handle = W.CreateNamedPipeW(name, access | W.FILE_FLAG_OVERLAPPED | 0x80000, 8, 1, 65536, 65536, 0, null);
    if !valid_handle(stream.handle) return false, W.INVALID_HANDLE_VALUE;

    stream.overlapped.hEvent = W.CreateEventW(null, 1, 0, null);
    if !stream.overlapped.hEvent return false, W.INVALID_HANDLE_VALUE;

    security := W.SECURITY_ATTRIBUTES.{ cast(u32) size_of(W.SECURITY_ATTRIBUTES), null, 1 };
    client := W.CreateFileW(name, client_access, 0, *security, W.OPEN_EXISTING, 0, null);

    if !valid_handle(client) return false, W.INVALID_HANDLE_VALUE;

    connected := W.ConnectNamedPipe(stream.handle, stream.overlapped);
    if connected == 0 {
        code := W.GetLastError();
        if code == 997 {
            transferred: u32;
            connected = GetOverlappedResult(stream.handle, stream.overlapped, *transferred, 1);
        } else if code == 535 connected = 1;

        if connected == 0 {
            checked_close(client);
            return false, W.INVALID_HANDLE_VALUE;
        }
    }

    committed = true;

    return true, client;
}

reset_operation :: (stream: *Stream) {
    assert(!operation_state(stream).pending, "pipe operation is still pending");
    event := stream.overlapped.hEvent;

    stream.overlapped.* = .{};
    stream.overlapped.hEvent = event;
    assert(ResetEvent(event) != 0, "pipe event reset failed");
}

close_stream :: (stream: *Stream) {
    assert(!has_pending(stream), "cannot destroy a pipe with outstanding I/O");
    if valid_handle(stream.handle) checked_close(stream.handle);

    if stream.overlapped {
        if stream.overlapped.hEvent checked_close(stream.overlapped.hEvent);
        state := operation_state(stream);
        owner := state.allocator;
        owner.proc(.FREE, 0, size_of(Operation_State), state, owner.data);
    }

    stream.* = .{};
    stream.eof = true;
} @Releases

begin_read :: (stream: *Stream, buffer: [] u8) -> bool, s64 {
    if stream.eof || buffer.count == 0 return true, 0;
    if !valid_handle(stream.handle) return false, 0;

    reset_operation(stream);
    received: u32;
    ok := W.ReadFile(stream.handle, buffer.data, cast(u32) min(buffer.count, 0x7fffffff), null, stream.overlapped);

    if ok != 0 {
        if GetOverlappedResult(stream.handle, stream.overlapped, *received, 0) == 0 return false, 0;
        return true, received;
    }

    code := W.GetLastError();
    if code == 109 {
        stream.eof = true;
        return true, 0;
    }

    if code != 997 return false, 0;

    operation_state(stream).pending = true;

    return true, 0;
}

finish_read :: (stream: *Stream) -> bool, s64 {
    if !has_pending(stream) return true, 0;

    received: u32;
    ok := GetOverlappedResult(stream.handle, stream.overlapped, *received, 0);
    code := W.GetLastError();

    if ok == 0 && code == 996 {
        canceled := CancelIoEx(stream.handle, stream.overlapped);

        assert(canceled != 0 || W.GetLastError() == 1168, "pipe cancellation failed");
        // Cancellation is asynchronous: drain completion before returning the
        // caller's buffer, even when timeout wins the race with arriving bytes.
        ok = GetOverlappedResult(stream.handle, stream.overlapped, *received, 1);
        code = W.GetLastError();
    }

    operation_state(stream).pending = false;

    if ok != 0 return true, received;

    if code == 109 stream.eof = true;
    if code == 109 || code == 995 return true, 0;

    return false, 0;
}
