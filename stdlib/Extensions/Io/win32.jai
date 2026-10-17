// The Win32 and Winsock calls the Windows backends make, and the Windows versions of the `sys_*`
// calls of platform.jai. Private to Io: the declarations are named `win_*` so that they never
// collide with anything the program imports, and they take `s64` for HANDLE and SOCKET (both are
// pointer sized, and -1 is INVALID_HANDLE_VALUE and INVALID_SOCKET).
//
// Everything here returns a result of 0 or more, or the negative Win32/Winsock error number, like
// the POSIX versions return the negative errno.

#if OS == .WINDOWS {
    // `kernel32` is declared in stacks.jai.
    ws2_32 :: #system_library "ws2_32";

    // ------------------------------------------------------------------------------------------
    // kernel32

    win_create_port :: (file: s64, port: s64, key: u64, threads: u32) -> s64 #foreign kernel32 "CreateIoCompletionPort";
    win_get_completions :: (port: s64, entries: *Overlapped_Entry, count: u32, removed: *u32, milliseconds: u32, alertable: s32) -> s32 #foreign kernel32 "GetQueuedCompletionStatusEx";
    win_post_completion :: (port: s64, bytes: u32, key: u64, overlapped: *Overlapped) -> s32 #foreign kernel32 "PostQueuedCompletionStatus";
    win_cancel_io :: (handle: s64, overlapped: *Overlapped) -> s32 #foreign kernel32 "CancelIoEx";
    win_create_file :: (name: *u16, access: u32, share: u32, security: *void, creation: u32, flags: u32, template: s64) -> s64 #foreign kernel32 "CreateFileW";
    win_read_file :: (handle: s64, buffer: *void, count: u32, transferred: *u32, overlapped: *Overlapped) -> s32 #foreign kernel32 "ReadFile";
    win_write_file :: (handle: s64, buffer: *void, count: u32, transferred: *u32, overlapped: *Overlapped) -> s32 #foreign kernel32 "WriteFile";
    win_close_handle :: (handle: s64) -> s32 #foreign kernel32 "CloseHandle";
    win_file_size :: (handle: s64, size: *s64) -> s32 #foreign kernel32 "GetFileSizeEx";
    win_last_error :: () -> u32 #foreign kernel32 "GetLastError";
    win_utf8_to_wide :: (code_page: u32, flags: u32, text: *u8, bytes: s32, wide: *u16, wide_count: s32) -> s32 #foreign kernel32 "MultiByteToWideChar";
    win_file_attributes :: (name: *u16) -> u32 #foreign kernel32 "GetFileAttributesW";
    win_processor_count :: (group: u16) -> u32 #foreign kernel32 "GetActiveProcessorCount";
    win_get_env :: (name: *u8, buffer: *u8, size: u32) -> u32 #foreign kernel32 "GetEnvironmentVariableA";
    win_load_library :: (name: *u8) -> s64 #foreign kernel32 "LoadLibraryA";
    win_module_handle :: (name: *u8) -> s64 #foreign kernel32 "GetModuleHandleA";
    win_proc_address :: (module: s64, name: *u8) -> *void #foreign kernel32 "GetProcAddress";
    win_create_event :: (security: *void, manual_reset: s32, initial: s32, name: *u16) -> s64 #foreign kernel32 "CreateEventW";
    win_set_event :: (event: s64) -> s32 #foreign kernel32 "SetEvent";
    win_wait_for_one :: (handle: s64, milliseconds: u32) -> u32 #foreign kernel32 "WaitForSingleObject";
    win_wait_for_many :: (count: u32, handles: *s64, wait_all: s32, milliseconds: u32) -> u32 #foreign kernel32 "WaitForMultipleObjects";
    win_module_file_name :: (module: s64, buffer: *u16, size: u32) -> u32 #foreign kernel32 "GetModuleFileNameW";
    win_sleep :: (milliseconds: u32) #foreign kernel32 "Sleep";

    // ------------------------------------------------------------------------------------------
    // Winsock

    win_wsa_startup :: (version: u16, data: *Wsa_Data) -> s32 #foreign ws2_32 "WSAStartup";
    win_wsa_cleanup :: () -> s32 #foreign ws2_32 "WSACleanup";
    win_wsa_error :: () -> s32 #foreign ws2_32 "WSAGetLastError";
    win_socket :: (family: s32, kind: s32, protocol: s32, info: *void, group: u32, flags: u32) -> s64 #foreign ws2_32 "WSASocketW";
    win_close_socket :: (socket: s64) -> s32 #foreign ws2_32 "closesocket";
    win_bind :: (socket: s64, address: *void, length: s32) -> s32 #foreign ws2_32 "bind";
    win_listen :: (socket: s64, backlog: s32) -> s32 #foreign ws2_32 "listen";
    win_socket_name :: (socket: s64, address: *void, length: *s32) -> s32 #foreign ws2_32 "getsockname";
    win_peer_name :: (socket: s64, address: *void, length: *s32) -> s32 #foreign ws2_32 "getpeername";
    win_set_socket_option :: (socket: s64, level: s32, name: s32, value: *void, length: s32) -> s32 #foreign ws2_32 "setsockopt";
    win_shutdown :: (socket: s64, how: s32) -> s32 #foreign ws2_32 "shutdown";
    win_ioctl :: (socket: s64, code: u32, in_buffer: *void, in_size: u32, out_buffer: *void, out_size: u32, returned: *u32, overlapped: *void, completion: *void) -> s32 #foreign ws2_32 "WSAIoctl";
    win_receive :: (socket: s64, buffers: *Wsa_Buffer, count: u32, received: *u32, flags: *u32, overlapped: *Overlapped, completion: *void) -> s32 #foreign ws2_32 "WSARecv";
    win_send :: (socket: s64, buffers: *Wsa_Buffer, count: u32, sent: *u32, flags: u32, overlapped: *Overlapped, completion: *void) -> s32 #foreign ws2_32 "WSASend";
    win_get_address_info :: (node: *u8, service: *u8, hints: *Addrinfo, result: **Addrinfo) -> s32 #foreign ws2_32 "getaddrinfo";
    win_free_address_info :: (result: *Addrinfo) #foreign ws2_32 "freeaddrinfo";

    // ------------------------------------------------------------------------------------------
    // Types and numbers

    // Room for the 408 bytes of WSADATA.
    Wsa_Data :: struct {
        words: [64] u64;
    }

    Wsa_Buffer :: struct {
        length: u32;
        data: *void;
    }

    // OVERLAPPED_ENTRY, what the completion port returns.
    Overlapped_Entry :: struct {
        key: u64;
        overlapped: *Overlapped;
        internal: u64;
        bytes: u32;
    }

    Guid :: struct {
        data1: u32;
        data2: u16;
        data3: u16;
        data4: [8] u8;
    }

    Accept_Ex :: #type (listener: s64, accepted: s64, buffer: *void, receive_length: u32, local_length: u32, remote_length: u32, received: *u32, overlapped: *Overlapped) -> s32 #c_call;
    Connect_Ex :: #type (socket: s64, address: *void, length: s32, send_buffer: *void, send_length: u32, sent: *u32, overlapped: *Overlapped) -> s32 #c_call;
    Status_To_Error :: #type (status: s32) -> u32 #c_call;

    INFINITE :: 0xffff_ffff;
    WAIT_TIMEOUT :: 258;
    ERROR_TIMEOUT :: 1460;
    WAIT_OBJECT_0 :: 0;

    ERROR_HANDLE_EOF :: 38;
    ERROR_IO_PENDING :: 997;
    ERROR_NOT_FOUND :: 1168;

    GENERIC_READ :: 0x8000_0000;
    GENERIC_WRITE :: 0x4000_0000;
    FILE_APPEND_DATA :: 0x4;
    FILE_SHARE_ALL :: 0x7;
    CREATE_NEW :: 1;
    CREATE_ALWAYS :: 2;
    OPEN_EXISTING :: 3;
    OPEN_ALWAYS :: 4;
    FILE_ATTRIBUTE_NORMAL :: 0x80;
    FILE_ATTRIBUTE_DIRECTORY :: 0x10;
    FILE_FLAG_OVERLAPPED :: 0x4000_0000;
    CP_UTF8 :: 65001;

    WSA_FLAG_OVERLAPPED :: 0x1;
    WSA_FLAG_NO_HANDLE_INHERIT :: 0x80;
    SIO_GET_EXTENSION_FUNCTION_POINTER :: 0xC800_0006;
    SO_EXCLUSIVEADDRUSE :: -5;
    SO_UPDATE_ACCEPT_CONTEXT :: 0x700B;
    SO_UPDATE_CONNECT_CONTEXT :: 0x7010;

    // Completion keys of the packets that belong to no operation.
    KEY_OPERATION :: 0;
    KEY_WAKE :: 1;
    KEY_RING :: 2;

    // The most a single WSASend takes. A send of a large buffer only completes when the peer has
    // taken all of it, and one that is canceled half way may report no progress; in pieces of a
    // socket buffer's size the caller can count what went out.
    SEND_CHUNK :: 1 << 16;

    // The length AcceptEx wants reserved for each of the two addresses it writes.
    ACCEPT_ADDRESS_BYTES :: 44;

    win_error :: () -> s64 {
        return win_last_error();
    }

    // Starts Winsock for this process (reference counted; `win_sockets_stop` undoes it).
    win_sockets_start :: () -> bool {
        data: Wsa_Data;

        return win_wsa_startup(0x0202, *data) == 0;
    }

    win_sockets_stop :: () {
        win_wsa_cleanup();
    }

    // The extension function `guid` names, for a socket of this provider, or null.
    win_extension :: (socket: s64, guid: Guid) -> *void {
        copy := guid;
        function: *void;
        returned: u32;

        result := win_ioctl(socket, SIO_GET_EXTENSION_FUNCTION_POINTER, *copy, size_of(Guid), *function, 8, *returned, null, null);

        if result != 0 return null;

        return function;
    }

    guid_accept_ex :: () -> Guid {
        guid: Guid;

        guid.data1 = 0xB5367DF1;
        guid.data2 = 0xCBAC;
        guid.data3 = 0x11CF;
        guid.data4[0] = 0x95;
        guid.data4[1] = 0xCA;
        guid.data4[2] = 0x00;
        guid.data4[3] = 0x80;
        guid.data4[4] = 0x5F;
        guid.data4[5] = 0x48;
        guid.data4[6] = 0xA1;
        guid.data4[7] = 0x92;

        return guid;
    }

    guid_connect_ex :: () -> Guid {
        guid: Guid;

        guid.data1 = 0x25A207B9;
        guid.data2 = 0xDDF3;
        guid.data3 = 0x4660;
        guid.data4[0] = 0x8E;
        guid.data4[1] = 0xE9;
        guid.data4[2] = 0x76;
        guid.data4[3] = 0xE5;
        guid.data4[4] = 0x8C;
        guid.data4[5] = 0x74;
        guid.data4[6] = 0x06;
        guid.data4[7] = 0x3E;

        return guid;
    }

    // The Win32 error behind an NTSTATUS, the way a completion packet reports failures.
    win_status_error :: (status: s32) -> s64 {
        if status >= 0 return 0;

        ntdll := win_module_handle("ntdll.dll");
        function := cast(Status_To_Error) win_proc_address(ntdll, "RtlNtStatusToDosError");

        if !function return E_INVAL;

        return cast(s64) function(status);
    }

    // Win32 errors with no `E_*` constant of their own that mean one of the portable errors.
    windows_error :: (code: s64) -> Io_Error {
        if code == {
            case 3; return .NOT_FOUND;
            case 183; return .ALREADY_EXISTS;
            case 32; return .PERMISSION_DENIED;
            case 39; return .NO_SPACE;
            case 123; return .INVALID;
            case 161; return .INVALID;
            case 206; return .INVALID;
            case 64; return .CONNECTION_RESET;
            case 232; return .BROKEN_PIPE;
            case 121; return .TIMED_OUT;
            case 1225; return .CONNECTION_REFUSED;
            case 1229; return .NOT_CONNECTED;
            case 1230; return .CONNECTION_RESET;
            case 1231; return .NETWORK_UNREACHABLE;
            case 1232; return .HOST_UNREACHABLE;
            case 1234; return .CONNECTION_REFUSED;
            case 1236; return .CONNECTION_ABORTED;
            case 10013; return .PERMISSION_DENIED;
            case 10022; return .INVALID;
            case 10024; return .TOO_MANY_OPEN_FILES;
            case 10050; return .NETWORK_UNREACHABLE;
            case 10052; return .CONNECTION_RESET;
            case 10058; return .BROKEN_PIPE;
        }

        return .OTHER;
    }

    // ------------------------------------------------------------------------------------------
    // The sys_* calls

    env_text :: (name: *u8, buffer: *[256] u8) -> string {
        length := win_get_env(name, buffer.data, 255);
        if length == 0 || length > 255 return "";

        return string.{ length, buffer.data };
    }

    // A socket that completion packets will be reported for: overlapped, not inherited, and tied
    // to the running scheduler's port.
    sys_socket :: (family: s32, nonblocking: bool) -> s64 {
        socket := win_socket(family, SOCK_STREAM, IPPROTO_TCP, null, 0, WSA_FLAG_OVERLAPPED | WSA_FLAG_NO_HANDLE_INHERIT);
        if socket == -1 return -cast(s64) win_wsa_error();

        port := current_scheduler().engine.iocp.port;

        if win_create_port(socket, port, KEY_OPERATION, 0) == 0 {
            error := win_error();
            win_close_socket(socket);

            return -error;
        }

        return socket;
    }

    wsa_checked :: (result: s32) -> s64 {
        if result != 0 return -cast(s64) win_wsa_error();

        return 0;
    }

    sys_set_option :: (fd: s64, level: s32, name: s32, value: s32) -> s64 {
        copy := value;

        return wsa_checked(win_set_socket_option(fd, level, name, *copy, 4));
    }

    // Windows has no SO_REUSEPORT, and its SO_REUSEADDR lets other programs take the port over. A
    // plain listener therefore asks for the opposite (exclusive use); one that wants to share the
    // port between threads uses SO_REUSEADDR, which on Windows does not balance connections: the
    // system hands them to one of the listeners.
    sys_prepare_listener :: (fd: s64, reuse_port: bool) -> s64 {
        if reuse_port return sys_set_option(fd, SOL_SOCKET, SO_REUSEADDR, 1);

        return sys_set_option(fd, SOL_SOCKET, SO_EXCLUSIVEADDRUSE, 1);
    }

    sys_bind :: (fd: s64, address: *Sock_Storage, length: u32) -> s64 {
        return wsa_checked(win_bind(fd, address, cast(s32) length));
    }

    sys_listen :: (fd: s64, backlog: s32) -> s64 {
        return wsa_checked(win_listen(fd, backlog));
    }

    sys_socket_name :: (fd: s64, address: *Sock_Storage, peer: bool) -> s64 {
        length: s32 = size_of(Sock_Storage);

        if peer return wsa_checked(win_peer_name(fd, address, *length));

        return wsa_checked(win_socket_name(fd, address, *length));
    }

    sys_shutdown :: (fd: s64, how: s32) -> s64 {
        return wsa_checked(win_shutdown(fd, how));
    }

    sys_socket_error :: (fd: s64) -> s64 {
        return E_UNSUPPORTED;
    }

    // Opens `path` (UTF-8) for overlapped IO. The POSIX flags of files.jai are translated.
    sys_open :: (path: *u8, flags: s32, mode: u32) -> s64 {
        length := 0;
        while path[length] length += 1;

        wide := cast(*u16) alloc((length + 1) * 2);

        defer free(wide);

        converted := win_utf8_to_wide(CP_UTF8, 0, path, cast(s32) length, wide, cast(s32) length);
        if converted <= 0 && length > 0 return -E_INVAL;

        wide[converted] = 0;

        access: u32;
        write := (flags & 3) != O_RDONLY;

        if (flags & 3) == O_RDWR access = GENERIC_READ | GENERIC_WRITE;
        else if (flags & O_APPEND) access = FILE_APPEND_DATA;
        else if write access = GENERIC_WRITE;
        else access = GENERIC_READ;

        creation: u32 = OPEN_EXISTING;

        if flags & O_CREAT {
            if flags & O_EXCL creation = CREATE_NEW;
            else if flags & O_TRUNC creation = CREATE_ALWAYS;
            else creation = OPEN_ALWAYS;
        }

        handle := win_create_file(wide, access, FILE_SHARE_ALL, null, creation, FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED, 0);

        if handle == -1 {
            error := win_error();

            // Opening a directory for writing is "access denied" here; say what is wrong.
            if error == E_ACCES {
                attributes := win_file_attributes(wide);
                if attributes != 0xffff_ffff && (attributes & FILE_ATTRIBUTE_DIRECTORY) != 0 return -E_ISDIR;
            }

            return -error;
        }

        // With an IoRing the file is not tied to the port: the ring reports on its own.
        iocp := current_scheduler().engine.iocp;

        if iocp.ring == 0 {
            if win_create_port(handle, iocp.port, KEY_OPERATION, 0) == 0 {
                error := win_error();
                win_close_handle(handle);

                return -error;
            }
        }

        return handle;
    }

    sys_close :: (fd: s64) -> s64 {
        if win_close_handle(fd) == 0 return -win_error();

        return 0;
    } @Releases

    // Closes a socket or file. Operations still under way on it complete with the cancel error.
    sys_close_handle :: (handle: Handle) -> s64 {
        win_cancel_io(handle.fd, null);

        if handle.kind == .SOCKET {
            if win_close_socket(handle.fd) != 0 return -cast(s64) win_wsa_error();

            return 0;
        }

        return sys_close(handle.fd);
    }

    sys_processors :: () -> s64 {
        count := win_processor_count(0xffff);
        if count < 1 return 1;

        return count;
    }

    sys_file_size :: (fd: s64) -> s64 {
        size: s64;

        if win_file_size(fd, *size) == 0 return -win_error();

        return size;
    }

    // POSIX-only calls, which the readiness backends make and no Windows backend ever reaches.

    sys_pipe :: (fds: *[2] s32, nonblocking: bool) -> s64 {
        return -E_UNSUPPORTED;
    }

    sys_accept :: (fd: s64, address: *Sock_Storage, length: *u32) -> s64 {
        return -E_UNSUPPORTED;
    }

    sys_connect :: (fd: s64, address: *Sock_Storage, length: u32) -> s64 {
        return -E_UNSUPPORTED;
    }

    sys_recv :: (fd: s64, buffer: *void, length: s64) -> s64 {
        return -E_UNSUPPORTED;
    }

    sys_peek :: (fd: s64) -> s64 {
        return -E_UNSUPPORTED;
    }

    sys_send :: (fd: s64, buffer: *void, length: s64) -> s64 {
        return -E_UNSUPPORTED;
    }

    sys_read :: (fd: s64, buffer: *void, length: s64, offset: s64) -> s64 {
        return -E_UNSUPPORTED;
    }

    sys_write :: (fd: s64, buffer: *void, length: s64, offset: s64) -> s64 {
        return -E_UNSUPPORTED;
    }
}
