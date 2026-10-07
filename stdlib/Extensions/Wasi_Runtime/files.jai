// Files and directories: the C calls the stdlib's `OS == .WASM` code makes (`open`, `fopen`,
// `stat`, `opendir`, `rename`, `realpath`...), on top of WASI preview 1's descriptor and path
// calls. WASI has no global file system: the host hands the program pre-opened directories, and
// every path call names one of them plus a path relative to it. Paths here are made absolute
// against the working directory, cleaned of `.` and `..`, and given to the pre-opened directory
// whose name is their longest prefix. The working directory starts as `$PWD` when a pre-opened
// directory covers it (tools/wasi_run.mjs pre-opens `/` and sets `PWD`), else `/`.
//
// Results use the Linux x86-64 layouts (`struct stat`, `struct dirent`) and errno numbers that
// the stdlib's `OS == .WASM` code shares with the interpreter's sandbox host. WASI has no
// permission bits or owners: `stat` reports 0644 for files and 0755 for directories, and
// `chmod` only checks that the path exists.

#scope_module

wasi_fd_close :: (fd: s32) -> s32 #foreign wasi "fd_close";
wasi_fd_seek :: (fd: s32, offset: s64, whence: s32, position: u32) -> s32 #foreign wasi "fd_seek";
wasi_fd_filestat_get :: (fd: s32, result: u32) -> s32 #foreign wasi "fd_filestat_get";
wasi_fd_readdir :: (fd: s32, buffer: u32, buffer_bytes: u32, cookie: u64, used: u32) -> s32 #foreign wasi "fd_readdir";
wasi_fd_fdstat_get :: (fd: s32, result: u32) -> s32 #foreign wasi "fd_fdstat_get";
wasi_fd_prestat_get :: (fd: s32, result: u32) -> s32 #foreign wasi "fd_prestat_get";
wasi_fd_prestat_dir_name :: (fd: s32, name: u32, name_bytes: u32) -> s32 #foreign wasi "fd_prestat_dir_name";
wasi_path_open :: (directory: s32, lookup: s32, path: u32, path_bytes: u32, open_flags: s32, rights: u64, inherited_rights: u64, fd_flags: s32, fd: u32) -> s32 #foreign wasi "path_open";
wasi_path_filestat_get :: (directory: s32, lookup: s32, path: u32, path_bytes: u32, result: u32) -> s32 #foreign wasi "path_filestat_get";
wasi_path_create_directory :: (directory: s32, path: u32, path_bytes: u32) -> s32 #foreign wasi "path_create_directory";
wasi_path_remove_directory :: (directory: s32, path: u32, path_bytes: u32) -> s32 #foreign wasi "path_remove_directory";
wasi_path_unlink_file :: (directory: s32, path: u32, path_bytes: u32) -> s32 #foreign wasi "path_unlink_file";
wasi_path_rename :: (from_directory: s32, from: u32, from_bytes: u32, to_directory: s32, to: u32, to_bytes: u32) -> s32 #foreign wasi "path_rename";
wasi_path_link :: (from_directory: s32, lookup: s32, from: u32, from_bytes: u32, to_directory: s32, to: u32, to_bytes: u32) -> s32 #foreign wasi "path_link";
wasi_path_readlink :: (directory: s32, path: u32, path_bytes: u32, buffer: u32, buffer_bytes: u32, used: u32) -> s32 #foreign wasi "path_readlink";

// Linux errno values (the ones the runtime reports itself; WASI's own codes go through
// `errno_from_wasi`).
EPERM :: 1;
EIO :: 5;
EBADF :: 9;
ENOMEM :: 12;
EACCES :: 13;
EEXIST :: 17;
ENOTDIR :: 20;
EISDIR :: 21;
ENAMETOOLONG :: 36;
ENOSYS :: 38;
ELOOP :: 40;

// Linux x86-64 <fcntl.h> flags.
O_ACCESS_MODE :: 3;
O_WRONLY :: 1;
O_RDWR :: 2;
O_CREAT :: 0x40;
O_EXCL :: 0x80;
O_TRUNC :: 0x200;
O_APPEND :: 0x400;
O_DIRECTORY :: 0x10000;
O_NOFOLLOW :: 0x20000;

PATH_BYTES :: 4096;

// WASI rights: a descriptor gets those its pre-opened directory passes on (asking for more
// fails the open), minus the reading or the writing ones when it is opened only for the other
// (node's WASI picks the host open mode from the rights asked for).
RIGHTS_READING: u64 : (1 << 1) | (1 << 14); // fd_read, fd_readdir
RIGHTS_WRITING: u64 : (1 << 0) | (1 << 6) | (1 << 8) | (1 << 22); // fd_datasync, fd_write, fd_allocate, fd_filestat_set_size

// WASI's `filestat`.
Wasi_File_Status :: struct {
    device: u64;
    inode: u64;
    kind: u8;
    kind_padding: [7] u8;
    links: u64;
    size: u64;
    accessed: u64;
    modified: u64;
    changed: u64;
}

#assert size_of(Wasi_File_Status) == 64;

// Linux x86-64 `struct stat`.
Linux_File_Status :: struct {
    device: u64;
    inode: u64;
    links: u64;
    mode: u32;
    user: u32;
    group: u32;
    padding: u32;
    special_device: u64;
    size: s64;
    block_size: s64;
    blocks: s64;
    accessed: Seconds_And_Nanoseconds;
    modified: Seconds_And_Nanoseconds;
    changed: Seconds_And_Nanoseconds;
    reserved: [3] s64;
}

#assert size_of(Linux_File_Status) == 144;

// Linux `struct dirent`.
Linux_Directory_Entry :: struct {
    inode: u64;
    next_offset: s64;
    record_bytes: u16;
    kind: u8;
    name: [256] u8;
}

// One pre-opened directory, its name cleaned into an absolute path.
Preopened_Directory :: struct {
    fd: s32;
    inherited_rights: u64;
    absolute: bool; // the host named it with a leading `/`
    name_bytes: s64;
    name: [PATH_BYTES] u8;
}

MAX_PREOPENED :: 16;
preopened: [MAX_PREOPENED] Preopened_Directory;
preopened_count: s64;
files_ready: bool;

working_directory: [PATH_BYTES] u8;
working_directory_bytes: s64;

// Scratch for the paths of one call; `rename` and `link` take two.
first_path: [PATH_BYTES] u8;
second_path: [PATH_BYTES] u8;
link_target: [PATH_BYTES] u8;

errno_from_wasi :: (code: s32) -> s32 #no_context {
    if code == {
        case 1; return 7; // 2big
        case 2; return EACCES;
        case 6; return EAGAIN;
        case 8; return EBADF;
        case 10; return 16; // busy
        case 16; return EDEADLK;
        case 20; return EEXIST;
        case 22; return 27; // file too big
        case 27; return 4; // interrupted
        case 28; return EINVAL;
        case 29; return EIO;
        case 31; return EISDIR;
        case 32; return ELOOP;
        case 33; return 24; // too many open files
        case 34; return 31; // too many links
        case 37; return ENAMETOOLONG;
        case 44; return ENOENT;
        case 48; return ENOMEM;
        case 51; return 28; // no space
        case 52; return ENOSYS;
        case 54; return ENOTDIR;
        case 55; return 39; // not empty
        case 58; return 95; // not supported
        case 63; return EPERM;
        case 68; return ERANGE;
        case 69; return 30; // read-only file system
        case 70; return 29; // not seekable
        case 71; return ESRCH;
        case 73; return ETIMEDOUT;
        case 75; return 18; // cross-device
        case 76; return EACCES; // a right the descriptor lacks
    }
    return EIO;
}

// Records a failed WASI call's error and returns -1.
failed :: (code: s32) -> s32 #no_context {
    error_number = errno_from_wasi(code);
    return -1;
}

// Appends the components of `path` to the clean absolute path `into[0..length-1]`: empty and
// `.` components are dropped, `..` removes the last one (by name, as the path is not resolved
// through symbolic links). Returns the new length, -1 when it does not fit.
append_components :: (into: *u8, length: s64, path: *u8, path_bytes: s64) -> s64 #no_context {
    end := length;
    at := 0;
    while at < path_bytes {
        while at < path_bytes && path[at] == #char "/" at += 1;
        start := at;
        while at < path_bytes && path[at] != #char "/" at += 1;
        count := at - start;
        if count == 0 || (count == 1 && path[start] == #char ".") continue;
        if count == 2 && path[start] == #char "." && path[start + 1] == #char "." {
            while end > 1 && into[end - 1] != #char "/" end -= 1;
            if end > 1 end -= 1;
            continue;
        }
        if end + count + 2 > PATH_BYTES return -1;
        if end > 1 {
            into[end] = #char "/";
            end += 1;
        }
        for 0..count - 1 into[end + it] = path[start + it];
        end += count;
    }
    into[end] = 0;
    return end;
}

// `path` made absolute against the working directory and cleaned, in `into`; its length, or -1
// (errno set) when it is empty or too long.
absolute_path :: (path: *u8, into: *u8) -> s64 #no_context {
    if !path || !path[0] {
        error_number = ENOENT;
        return -1;
    }
    length: s64 = 1;
    into[0] = #char "/";
    if path[0] != #char "/" {
        for 0..working_directory_bytes - 1 into[it] = working_directory[it];
        length = working_directory_bytes;
    }
    length = append_components(into, length, path, c_length(path));
    if length < 0 error_number = ENAMETOOLONG;
    return length;
}

// Finds the pre-opened directories and the starting working directory, once.
prepare_files :: () #no_context {
    if files_ready return;
    files_ready = true;
    // Pre-opened descriptors are numbered from 3 up, until the first that is not one.
    fd: s32 = 3;
    while preopened_count < MAX_PREOPENED {
        status: [2] u32; // tag (0: directory), name length
        if wasi_fd_prestat_get(fd, offset32(status.data)) != 0 break;
        if (status[0] & 0xff) == 0 && status[1] < PATH_BYTES {
            raw := link_target.data;
            // WASI's `fdstat`: kind, flags, then the rights and the inherited rights.
            descriptor: [3] u64;
            if wasi_fd_prestat_dir_name(fd, offset32(raw), status[1]) == 0 && wasi_fd_fdstat_get(fd, offset32(descriptor.data)) == 0 {
                entry := *preopened[preopened_count];
                entry.fd = fd;
                entry.inherited_rights = descriptor[2];
                entry.absolute = status[1] > 0 && raw[0] == #char "/";
                // A relative name (`.` from `wasmtime --dir .`) is taken from the root.
                entry.name[0] = #char "/";
                entry.name_bytes = append_components(entry.name.data, 1, raw, cast(s64) status[1]);
                if entry.name_bytes > 0 preopened_count += 1;
            }
        }
        fd += 1;
    }
    working_directory[0] = #char "/";
    working_directory[1] = 0;
    working_directory_bytes = 1;
    start := wasi_getenv("PWD");
    if start && start[0] == #char "/" {
        first_path[0] = #char "/";
        length := append_components(first_path.data, 1, start, c_length(start));
        index := preopened_for(first_path.data, length);
        if length > 0 && index >= 0 && preopened[index].absolute {
            for 0..length working_directory[it] = first_path[it];
            working_directory_bytes = length;
        }
    }
}

// The pre-opened directory whose name is the longest prefix of the clean absolute `path`, or -1.
preopened_for :: (path: *u8, length: s64) -> s64 #no_context {
    best := -1;
    best_bytes := -1;
    for i: 0..preopened_count - 1 {
        entry := *preopened[i];
        count := entry.name_bytes;
        matches := count == 1 || (length >= count && bytes_equal(path, entry.name.data, count) && (length == count || path[count] == #char "/"));
        if matches && count > best_bytes {
            best = i;
            best_bytes = count;
        }
    }
    return best;
}

// A path as WASI takes it: a pre-opened directory and a path relative to it.
Wasi_Path :: struct {
    directory: s32 = -1;
    rights: u64; // what the directory passes on to what is opened in it
    path: u32;
    bytes: u32;
}

// Resolves `path` into `scratch` (PATH_BYTES long). `directory` is -1, with errno set, on failure.
wasi_path :: (path: *u8, scratch: *u8) -> Wasi_Path #no_context {
    prepare_files();
    result: Wasi_Path;
    length := absolute_path(path, scratch);
    if length < 0 return result;
    index := preopened_for(scratch, length);
    if index < 0 {
        error_number = EACCES;
        return result;
    }
    prefix := preopened[index].name_bytes;
    start := ifx prefix == 1 then 1 else prefix + 1;
    result.directory = preopened[index].fd;
    result.rights = preopened[index].inherited_rights;
    if start >= length {
        // The pre-opened directory itself.
        scratch[0] = #char ".";
        scratch[1] = 0;
        result.path = offset32(scratch);
        result.bytes = 1;
    } else {
        result.path = offset32(scratch + start);
        result.bytes = cast,no_check(u32) (length - start);
    }
    return result;
}

LOOKUP_FOLLOW :: 1;
WASI_EINVAL :: 28;

path_status :: (path: *u8, follow: bool, status: *Wasi_File_Status) -> s32 #no_context {
    target := wasi_path(path, first_path.data);
    if target.directory < 0 return -1;
    code := wasi_path_filestat_get(target.directory, ifx follow then LOOKUP_FOLLOW else 0, target.path, target.bytes, offset32(status));
    if code != 0 return failed(code);
    return 0;
}

// Fills a Linux `struct stat` from WASI's `filestat`.
linux_status :: (status: *Wasi_File_Status, result: *Linux_File_Status) #no_context {
    memset(result, 0, size_of(Linux_File_Status));
    // Type bits as in Linux <sys/stat.h>; permissions rw-r--r-- (0x1a4) or rwxr-xr-x (0x1ed).
    mode: u32;
    if status.kind == {
        case 1; mode = 0x6000 | 0x1b0; // block device, rw-rw----
        case 2; mode = 0x2000 | 0x1b6; // character device, rw-rw-rw-
        case 3; mode = 0x4000 | 0x1ed; // directory
        case 5; mode = 0xc000 | 0x1ed; // datagram socket
        case 6; mode = 0xc000 | 0x1ed; // stream socket
        case 7; mode = 0xa000 | 0x1ff; // symbolic link, rwxrwxrwx
        case; mode = 0x8000 | 0x1a4; // regular file (4), or unknown
    }
    result.device = status.device;
    result.inode = status.inode;
    result.links = status.links;
    result.mode = mode;
    result.size = cast,no_check(s64) status.size;
    result.block_size = 4096;
    result.blocks = cast,no_check(s64) ((status.size + 511) / 512);
    result.accessed = time_from_nanoseconds(status.accessed);
    result.modified = time_from_nanoseconds(status.modified);
    result.changed = time_from_nanoseconds(status.changed);
}

time_from_nanoseconds :: (nanoseconds: u64) -> Seconds_And_Nanoseconds #no_context {
    result: Seconds_And_Nanoseconds;
    result.seconds = cast,no_check(s64) (nanoseconds / 1_000_000_000);
    result.nanoseconds = cast,no_check(s64) (nanoseconds % 1_000_000_000);
    return result;
}

// Opens `path` with Linux `open` flags; the descriptor, or -1 with errno set.
open_path :: (path: *u8, flags: s32) -> s32 #no_context {
    target := wasi_path(path, first_path.data);
    if target.directory < 0 return -1;
    open_flags: s32;
    if (flags & O_CREAT) != 0 open_flags |= 1;
    if (flags & O_DIRECTORY) != 0 open_flags |= 2;
    if (flags & O_EXCL) != 0 open_flags |= 4;
    if (flags & O_TRUNC) != 0 open_flags |= 8;
    fd_flags: s32 = ifx (flags & O_APPEND) != 0 then 1 else 0;
    rights := target.rights;
    access := flags & O_ACCESS_MODE;
    if access == 0 || (flags & O_DIRECTORY) != 0 rights &= ~RIGHTS_WRITING;
    else if access == O_WRONLY rights &= ~RIGHTS_READING;
    lookup: s32 = ifx (flags & O_NOFOLLOW) != 0 then 0 else LOOKUP_FOLLOW;
    fd: s32;
    code := wasi_path_open(target.directory, lookup, target.path, target.bytes, open_flags, rights, target.rights, fd_flags, offset32(*fd));
    if code != 0 return failed(code);
    return fd;
}

// ---------------------------------------------------------------------------------------------
// Descriptors

// `open` is variadic in C: the mode comes as a pointer to the extra arguments, and WASI has no
// use for it.
#program_export "open"
wasi_open :: (path: *u8, flags: s32, extra: *void) -> s32 #c_call {
    return open_path(path, flags);
}

#program_export "close"
wasi_close :: (fd: s32) -> s32 #c_call {
    code := wasi_fd_close(fd);
    if code != 0 return failed(code);
    return 0;
}

#program_export "lseek"
wasi_lseek :: (fd: s32, offset: s64, whence: s32) -> s64 #c_call {
    position: u64;
    code := wasi_fd_seek(fd, offset, whence, offset32(*position));
    if code != 0 return failed(code);
    return cast,no_check(s64) position;
}

#program_export "fstat"
wasi_fstat :: (fd: s32, result: *Linux_File_Status) -> s32 #c_call {
    status: Wasi_File_Status;
    code := wasi_fd_filestat_get(fd, offset32(*status));
    if code != 0 return failed(code);
    linux_status(*status, result);
    return 0;
}

// ---------------------------------------------------------------------------------------------
// Paths

#program_export "stat"
wasi_stat :: (path: *u8, result: *Linux_File_Status) -> s32 #c_call {
    status: Wasi_File_Status;
    if path_status(path, true, *status) != 0 return -1;
    linux_status(*status, result);
    return 0;
}

#program_export "lstat"
wasi_lstat :: (path: *u8, result: *Linux_File_Status) -> s32 #c_call {
    status: Wasi_File_Status;
    if path_status(path, false, *status) != 0 return -1;
    linux_status(*status, result);
    return 0;
}

// Only existence can be checked: WASI reports no permissions.
#program_export "access"
wasi_access :: (path: *u8, mode: s32) -> s32 #c_call {
    status: Wasi_File_Status;
    return path_status(path, true, *status);
}

#program_export "chmod"
wasi_chmod :: (path: *u8, mode: u32) -> s32 #c_call {
    status: Wasi_File_Status;
    return path_status(path, true, *status);
}

#program_export "fchmod"
wasi_fchmod :: (fd: s32, mode: u32) -> s32 #c_call {
    status: Wasi_File_Status;
    code := wasi_fd_filestat_get(fd, offset32(*status));
    if code != 0 return failed(code);
    return 0;
}

#program_export "mkdir"
wasi_mkdir :: (path: *u8, mode: u32) -> s32 #c_call {
    target := wasi_path(path, first_path.data);
    if target.directory < 0 return -1;
    code := wasi_path_create_directory(target.directory, target.path, target.bytes);
    if code != 0 return failed(code);
    return 0;
}

#program_export "rmdir"
wasi_rmdir :: (path: *u8) -> s32 #c_call {
    target := wasi_path(path, first_path.data);
    if target.directory < 0 return -1;
    code := wasi_path_remove_directory(target.directory, target.path, target.bytes);
    if code != 0 return failed(code);
    return 0;
}

#program_export "unlink"
wasi_unlink :: (path: *u8) -> s32 #c_call {
    target := wasi_path(path, first_path.data);
    if target.directory < 0 return -1;
    code := wasi_path_unlink_file(target.directory, target.path, target.bytes);
    if code != 0 return failed(code);
    return 0;
}

#program_export "remove"
wasi_remove :: (path: *u8) -> s32 #c_call {
    status: Wasi_File_Status;
    if path_status(path, false, *status) != 0 return -1;
    if status.kind == 3 return wasi_rmdir(path);
    return wasi_unlink(path);
}

#program_export "rename"
wasi_rename :: (from: *u8, to: *u8) -> s32 #c_call {
    source := wasi_path(from, first_path.data);
    if source.directory < 0 return -1;
    destination := wasi_path(to, second_path.data);
    if destination.directory < 0 return -1;
    code := wasi_path_rename(source.directory, source.path, source.bytes, destination.directory, destination.path, destination.bytes);
    if code != 0 return failed(code);
    return 0;
}

#program_export "link"
wasi_link :: (from: *u8, to: *u8) -> s32 #c_call {
    source := wasi_path(from, first_path.data);
    if source.directory < 0 return -1;
    destination := wasi_path(to, second_path.data);
    if destination.directory < 0 return -1;
    code := wasi_path_link(source.directory, 0, source.path, source.bytes, destination.directory, destination.path, destination.bytes);
    if code != 0 return failed(code);
    return 0;
}

#program_export "getcwd"
wasi_getcwd :: (buffer: *u8, size: u64) -> *u8 #c_call {
    prepare_files();
    if !buffer || size < cast(u64) working_directory_bytes + 1 {
        error_number = ERANGE;
        return null;
    }
    for 0..working_directory_bytes - 1 buffer[it] = working_directory[it];
    buffer[working_directory_bytes] = 0;
    return buffer;
}

#program_export "chdir"
wasi_chdir :: (path: *u8) -> s32 #c_call {
    status: Wasi_File_Status;
    if path_status(path, true, *status) != 0 return -1;
    if status.kind != 3 {
        error_number = ENOTDIR;
        return -1;
    }
    length := absolute_path(path, second_path.data);
    if length < 0 return -1;
    for 0..length working_directory[it] = second_path[it];
    working_directory_bytes = length;
    return 0;
}

// The absolute path with every symbolic link resolved, by reading each prefix's link (WASI
// has no call that does it at once). `resolved` null means a new `malloc` block.
#program_export "realpath"
wasi_realpath :: (path: *u8, resolved: *u8) -> *u8 #c_call {
    prepare_files();
    current: [PATH_BYTES] u8;
    length := absolute_path(path, current.data);
    if length < 0 return null;
    links_followed := 0;
    // `checked` is how much of `current` is known to have no symbolic links.
    checked: s64 = 1;
    while checked < length {
        end := checked + 1;
        while end < length && current[end] != #char "/" end += 1;
        saved := current[end];
        current[end] = 0;
        target := wasi_path(current.data, first_path.data);
        current[end] = saved;
        if target.directory < 0 return null;
        used: u32;
        code := wasi_path_readlink(target.directory, target.path, target.bytes, offset32(link_target.data), PATH_BYTES - 1, offset32(*used));
        if code == 0 && used > 0 {
            links_followed += 1;
            if links_followed > 40 {
                error_number = ELOOP;
                return null;
            }
            // Splice the link's target in place of the component: rebuild from the target
            // (absolute, or relative to the component's directory), then the rest.
            rest: [PATH_BYTES] u8;
            rest_bytes := length - end;
            for 0..rest_bytes - 1 rest[it] = current[end + it];
            // `checked` ends the component's directory (1: the root).
            parent := ifx link_target[0] == #char "/" then 1 else checked;
            length = append_components(current.data, parent, link_target.data, cast(s64) used);
            if length >= 0 length = append_components(current.data, length, rest.data, rest_bytes);
            if length < 0 {
                error_number = ENAMETOOLONG;
                return null;
            }
            checked = 1;
            continue;
        }
        // 0 bytes or EINVAL: not a link. Anything else (ENOENT...) fails the call.
        if code != 0 && code != WASI_EINVAL {
            failed(code);
            return null;
        }
        checked = end;
    }
    status: Wasi_File_Status;
    if path_status(current.data, true, *status) != 0 return null;
    into := resolved;
    if !into {
        into = cast(*u8) wasi_malloc(cast(u64) length + 1);
        if !into {
            error_number = ENOMEM;
            return null;
        }
    }
    for 0..length - 1 into[it] = current[it];
    into[length] = 0;
    return into;
}

// ---------------------------------------------------------------------------------------------
// Directories

DIRECTORY_BUFFER_BYTES :: 4096;
WASI_ENTRY_HEADER_BYTES :: 24; // next cookie, inode, name length, kind

Directory_Stream :: struct {
    fd: s32;
    cookie: u64;
    filled: s64; // bytes of `buffer` the last fd_readdir wrote
    at: s64;
    finished: bool; // the last fd_readdir returned less than a full buffer
    entry: Linux_Directory_Entry;
    buffer: [DIRECTORY_BUFFER_BYTES] u8;
}

#program_export "opendir"
wasi_opendir :: (path: *u8) -> *Directory_Stream #c_call {
    fd := open_path(path, O_DIRECTORY);
    if fd < 0 return null;
    stream := cast(*Directory_Stream) wasi_malloc(size_of(Directory_Stream));
    if !stream {
        wasi_fd_close(fd);
        error_number = ENOMEM;
        return null;
    }
    memset(stream, 0, size_of(Directory_Stream));
    stream.fd = fd;
    return stream;
}

// WASI lists the directory from a cookie into a buffer; an entry cut off at the end of a full
// buffer is read again, from its cookie, by the next fill.
#program_export "readdir"
wasi_readdir :: (stream: *Directory_Stream) -> *Linux_Directory_Entry #c_call {
    if !stream {
        error_number = EBADF;
        return null;
    }
    refilled := false;
    while true {
        if stream.at + WASI_ENTRY_HEADER_BYTES <= stream.filled {
            header := stream.buffer.data + stream.at;
            name_bytes := cast(s64) (cast(*u32) (header + 16)).*;
            if stream.at + WASI_ENTRY_HEADER_BYTES + name_bytes <= stream.filled {
                stream.cookie = (cast(*u64) header).*;
                stream.at += WASI_ENTRY_HEADER_BYTES + name_bytes;
                entry := *stream.entry;
                memset(entry, 0, size_of(Linux_Directory_Entry));
                entry.inode = (cast(*u64) (header + 8)).*;
                entry.next_offset = cast,no_check(s64) stream.cookie;
                entry.record_bytes = size_of(Linux_Directory_Entry);
                entry.kind = directory_entry_kind(header[20]);
                if name_bytes > 255 name_bytes = 255;
                for 0..name_bytes - 1 entry.name[it] = header[WASI_ENTRY_HEADER_BYTES + it];
                return entry;
            }
        }
        if stream.finished || refilled return null;
        used: u32;
        code := wasi_fd_readdir(stream.fd, offset32(stream.buffer.data), DIRECTORY_BUFFER_BYTES, stream.cookie, offset32(*used));
        if code != 0 {
            failed(code);
            return null;
        }
        stream.filled = used;
        stream.at = 0;
        stream.finished = used < DIRECTORY_BUFFER_BYTES;
        refilled = true;
    }
    return null;
}

// WASI's file type as a Linux DT_* value.
directory_entry_kind :: (kind: u8) -> u8 #no_context {
    if kind == {
        case 1; return 6; // block device
        case 2; return 2; // character device
        case 3; return 4; // directory
        case 4; return 8; // regular file
        case 5; return 12; // socket
        case 6; return 12;
        case 7; return 10; // symbolic link
    }
    return 0;
}

#program_export "closedir"
wasi_closedir :: (stream: *Directory_Stream) -> s32 #c_call {
    if !stream {
        error_number = EBADF;
        return -1;
    }
    code := wasi_fd_close(stream.fd);
    wasi_free(stream);
    if code != 0 return failed(code);
    return 0;
}

// ---------------------------------------------------------------------------------------------
// Streams: a `FILE` is a descriptor and two flags. Nothing is buffered, so `fflush` has nothing
// to do and a stream's position is its descriptor's.

Stream :: struct {
    fd: s32;
    at_end: bool;
    had_error: bool;
}

#program_export "fopen"
wasi_fopen :: (path: *u8, mode: *u8) -> *Stream #c_call {
    if !mode || !mode[0] {
        error_number = EINVAL;
        return null;
    }
    update := false;
    exclusive := false;
    for 1..c_length(mode) - 1 {
        if mode[it] == #char "+" update = true;
        if mode[it] == #char "x" exclusive = true;
    }
    flags: s32;
    if mode[0] == {
        case #char "r"; flags = 0;
        case #char "w"; flags = O_WRONLY | O_CREAT | O_TRUNC;
        case #char "a"; flags = O_WRONLY | O_CREAT | O_APPEND;
        case;
            error_number = EINVAL;
            return null;
    }
    if update flags = (flags & ~O_ACCESS_MODE) | O_RDWR;
    if exclusive flags |= O_EXCL;
    fd := open_path(path, flags);
    if fd < 0 return null;
    stream := cast(*Stream) wasi_malloc(size_of(Stream));
    if !stream {
        wasi_fd_close(fd);
        error_number = ENOMEM;
        return null;
    }
    stream.* = .{ fd = fd };
    return stream;
}

#program_export "fclose"
wasi_fclose :: (stream: *Stream) -> s32 #c_call {
    if !stream {
        error_number = EBADF;
        return -1;
    }
    code := wasi_fd_close(stream.fd);
    wasi_free(stream);
    if code != 0 return failed(code);
    return 0;
}

#program_export "fread"
wasi_fread :: (buffer: *void, size: u64, count: u64, stream: *Stream) -> u64 #c_call {
    if !stream || size == 0 || count == 0 return 0;
    total := size * count;
    done: u64;
    while done < total {
        step := wasi_read(stream.fd, cast(*u8) buffer + done, total - done);
        if step < 0 {
            stream.had_error = true;
            break;
        }
        if step == 0 {
            stream.at_end = true;
            break;
        }
        done += cast(u64) step;
    }
    return done / size;
}

#program_export "fwrite"
wasi_fwrite :: (buffer: *void, size: u64, count: u64, stream: *Stream) -> u64 #c_call {
    if !stream || size == 0 || count == 0 return 0;
    total := size * count;
    done: u64;
    while done < total {
        step := wasi_write(stream.fd, cast(*u8) buffer + done, total - done);
        if step <= 0 {
            stream.had_error = true;
            break;
        }
        done += cast(u64) step;
    }
    return done / size;
}

#program_export "fseek"
wasi_fseek :: (stream: *Stream, offset: s64, whence: s32) -> s32 #c_call {
    if !stream {
        error_number = EBADF;
        return -1;
    }
    if wasi_lseek(stream.fd, offset, whence) < 0 return -1;
    stream.at_end = false;
    return 0;
}

#program_export "fseeko"
wasi_fseeko :: (stream: *Stream, offset: s64, whence: s32) -> s32 #c_call {
    return wasi_fseek(stream, offset, whence);
}

#program_export "ftell"
wasi_ftell :: (stream: *Stream) -> s64 #c_call {
    if !stream {
        error_number = EBADF;
        return -1;
    }
    return wasi_lseek(stream.fd, 0, 1);
}

#program_export "ftello"
wasi_ftello :: (stream: *Stream) -> s64 #c_call {
    return wasi_ftell(stream);
}

#program_export "rewind"
wasi_rewind :: (stream: *Stream) #c_call {
    if !stream return;
    wasi_fseek(stream, 0, 0);
    stream.had_error = false;
}

#program_export "feof"
wasi_feof :: (stream: *Stream) -> s32 #c_call {
    return ifx stream && stream.at_end then 1 else 0;
}

#program_export "ferror"
wasi_ferror :: (stream: *Stream) -> s32 #c_call {
    return ifx stream && stream.had_error then 1 else 0;
}

#program_export "clearerr"
wasi_clearerr :: (stream: *Stream) #c_call {
    if !stream return;
    stream.at_end = false;
    stream.had_error = false;
}

#program_export "fflush"
wasi_fflush :: (stream: *Stream) -> s32 #c_call {
    return 0;
}

#program_export "fileno"
wasi_fileno :: (stream: *Stream) -> s32 #c_call {
    if !stream {
        error_number = EBADF;
        return -1;
    }
    return stream.fd;
}

// ---------------------------------------------------------------------------------------------
// System

// One processor (WASI preview 1 has no threads) and 64 KiB wasm pages.
#program_export "sysconf"
wasi_sysconf :: (name: s32) -> s64 #c_call {
    if name == {
        case 30; return PAGE_BYTES; // _SC_PAGESIZE
        case 83; return 1; // _SC_NPROCESSORS_CONF
        case 84; return 1; // _SC_NPROCESSORS_ONLN
    }
    error_number = EINVAL;
    return -1;
}

#program_export "strerror"
wasi_strerror :: (code: s32) -> *u8 #c_call {
    text: string;
    if code == {
        case 0; text = "Success\0";
        case EPERM; text = "Operation not permitted\0";
        case ENOENT; text = "No such file or directory\0";
        case ESRCH; text = "No such process\0";
        case EIO; text = "Input/output error\0";
        case EBADF; text = "Bad file descriptor\0";
        case EAGAIN; text = "Resource temporarily unavailable\0";
        case ENOMEM; text = "Cannot allocate memory\0";
        case EACCES; text = "Permission denied\0";
        case EEXIST; text = "File exists\0";
        case 18; text = "Invalid cross-device link\0";
        case ENOTDIR; text = "Not a directory\0";
        case EISDIR; text = "Is a directory\0";
        case EINVAL; text = "Invalid argument\0";
        case 24; text = "Too many open files\0";
        case 28; text = "No space left on device\0";
        case 29; text = "Illegal seek\0";
        case 30; text = "Read-only file system\0";
        case ERANGE; text = "Numerical result out of range\0";
        case EDEADLK; text = "Resource deadlock avoided\0";
        case ENAMETOOLONG; text = "File name too long\0";
        case ENOSYS; text = "Function not implemented\0";
        case 39; text = "Directory not empty\0";
        case ELOOP; text = "Too many levels of symbolic links\0";
        case 95; text = "Operation not supported\0";
        case ETIMEDOUT; text = "Connection timed out\0";
        case; text = "Unknown error\0";
    }
    return text.data;
}
